//! Permission sandbox — enforces mod permissions at runtime.
//!
//! Every mod operation is checked against the mod's declared permissions.
//! Operations that require permissions the mod doesn't have are blocked.

use std::path::Path;

use crate::manifest::ModPermission;

/// Configuration for the mod sandbox.
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    /// Whether to enforce permissions strictly (true) or just log warnings (false)
    pub strict_mode: bool,
    /// Root directory that mods are allowed to read from
    pub allowed_read_root: std::path::PathBuf,
    /// Root directory that mods are allowed to write to
    pub allowed_write_root: std::path::PathBuf,
    /// Whether mods can access the network
    pub allow_network: bool,
    /// Whether mods can execute arbitrary code
    pub allow_code_execution: bool,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            strict_mode: true,
            allowed_read_root: std::path::PathBuf::from("."),
            allowed_write_root: std::path::PathBuf::from("./mods-data"),
            allow_network: false,
            allow_code_execution: false,
        }
    }
}

/// Errors that the sandbox can raise.
#[derive(Debug, Clone, thiserror::Error)]
pub enum SandboxError {
    #[error("permission denied: {permission} (required for {action})")]
    PermissionDenied { permission: String, action: String },
    #[error("path traversal detected: {path} escapes allowed root")]
    PathTraversal { path: String },
    #[error("network access blocked")]
    NetworkBlocked,
    #[error("code execution blocked")]
    CodeExecutionBlocked,
}

/// Check permissions for a mod.
pub struct PermissionChecker {
    config: SandboxConfig,
    permissions: Vec<ModPermission>,
}

impl PermissionChecker {
    /// Create a new permission checker for a mod.
    pub fn new(config: SandboxConfig, permissions: Vec<ModPermission>) -> Self {
        Self {
            config,
            permissions,
        }
    }

    /// Check if the mod has a specific permission.
    pub fn has_permission(&self, perm: ModPermission) -> bool {
        self.permissions.contains(&perm)
    }

    /// Check if a file read is allowed.
    pub fn check_read(&self, path: &Path) -> Result<(), SandboxError> {
        if !self.has_permission(ModPermission::ReadFiles) {
            return Err(SandboxError::PermissionDenied {
                permission: "ReadFiles".to_string(),
                action: format!("read {}", path.display()),
            });
        }

        // Check for path traversal
        let canonical = self.validate_path(path, &self.config.allowed_read_root)?;
        tracing::trace!("Read allowed: {}", canonical.display());
        Ok(())
    }

    /// Check if a file write is allowed.
    pub fn check_write(&self, path: &Path) -> Result<(), SandboxError> {
        if !self.has_permission(ModPermission::WriteFiles) {
            return Err(SandboxError::PermissionDenied {
                permission: "WriteFiles".to_string(),
                action: format!("write {}", path.display()),
            });
        }

        let canonical = self.validate_path(path, &self.config.allowed_write_root)?;
        tracing::trace!("Write allowed: {}", canonical.display());
        Ok(())
    }

    /// Check if network access is allowed.
    pub fn check_network(&self) -> Result<(), SandboxError> {
        if !self.config.allow_network && !self.has_permission(ModPermission::Network) {
            return Err(SandboxError::NetworkBlocked);
        }
        Ok(())
    }

    /// Check if code execution is allowed.
    pub fn check_execute(&self) -> Result<(), SandboxError> {
        if !self.config.allow_code_execution && !self.has_permission(ModPermission::ExecuteCode) {
            return Err(SandboxError::CodeExecutionBlocked);
        }
        Ok(())
    }

    /// Validate that a path doesn't escape the allowed root.
    fn validate_path(
        &self,
        path: &Path,
        allowed_root: &Path,
    ) -> Result<std::path::PathBuf, SandboxError> {
        let normalized = path.to_string_lossy();

        // Resolve the path relative to the allowed root
        let resolved = if path.is_absolute() {
            path.to_path_buf()
        } else {
            allowed_root.join(path)
        };

        // Check for path traversal: count .. components and ensure the
        // resolved path stays within the allowed root
        let resolved_str = resolved.to_string_lossy();
        let root_str = allowed_root.to_string_lossy();

        // If the path contains .., check that the canonicalized form
        // still starts with the allowed root
        if normalized.contains("..") {
            // Canonicalize the root (if it exists) for comparison
            let canonical_root =
                std::fs::canonicalize(allowed_root).unwrap_or_else(|_| allowed_root.to_path_buf());

            // Try to canonicalize the resolved path
            // If the file doesn't exist yet, canonicalize the parent and join
            let canonical_resolved = std::fs::canonicalize(&resolved).unwrap_or_else(|_| {
                if let Some(parent) = resolved.parent() {
                    if let Some(filename) = resolved.file_name() {
                        std::fs::canonicalize(parent)
                            .unwrap_or_else(|_| parent.to_path_buf())
                            .join(filename)
                    } else {
                        resolved.clone()
                    }
                } else {
                    resolved.clone()
                }
            });

            let canonical_root_str = canonical_root.to_string_lossy();
            let canonical_resolved_str = canonical_resolved.to_string_lossy();

            if !canonical_resolved_str.starts_with(&*canonical_root_str) {
                return Err(SandboxError::PathTraversal {
                    path: normalized.to_string(),
                });
            }
        }

        // Also block absolute paths that don't start with the root
        if path.is_absolute() && !resolved_str.starts_with(&*root_str) {
            // Allow if it's within the root after canonicalization
            let canonical_root =
                std::fs::canonicalize(allowed_root).unwrap_or_else(|_| allowed_root.to_path_buf());
            let canonical_resolved =
                std::fs::canonicalize(&resolved).unwrap_or_else(|_| resolved.clone());
            if !canonical_resolved
                .to_string_lossy()
                .starts_with(&*canonical_root.to_string_lossy())
            {
                return Err(SandboxError::PathTraversal {
                    path: normalized.to_string(),
                });
            }
        }

        Ok(resolved)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn checker(permissions: Vec<ModPermission>) -> PermissionChecker {
        PermissionChecker::new(SandboxConfig::default(), permissions)
    }

    #[test]
    fn test_read_permission_granted() {
        let c = checker(vec![ModPermission::ReadFiles]);
        assert!(c.check_read(Path::new("data/file.txt")).is_ok());
    }

    #[test]
    fn test_read_permission_denied() {
        let c = checker(vec![]);
        assert!(c.check_read(Path::new("data/file.txt")).is_err());
    }

    #[test]
    fn test_write_permission_granted() {
        let c = checker(vec![ModPermission::WriteFiles]);
        assert!(c.check_write(Path::new("save/data.json")).is_ok());
    }

    #[test]
    fn test_network_blocked() {
        let c = checker(vec![]);
        assert!(c.check_network().is_err());
    }

    #[test]
    fn test_network_allowed_with_permission() {
        let c = checker(vec![ModPermission::Network]);
        assert!(c.check_network().is_ok());
    }

    #[test]
    fn test_code_execution_blocked() {
        let c = checker(vec![]);
        assert!(c.check_execute().is_err());
    }

    #[test]
    fn test_path_traversal_detected() {
        let c = checker(vec![ModPermission::ReadFiles]);
        // Path with .. that escapes root
        let result = c.check_read(Path::new("../../etc/passwd"));
        assert!(result.is_err());
    }
}
