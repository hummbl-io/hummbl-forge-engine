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

        // BLUE TEAM: Canonicalize the root for comparison
        // If the root doesn't exist, normalize it lexically so that
        // `./mods-data` and `mods-data` compare equal.
        let canonical_root = std::fs::canonicalize(allowed_root)
            .unwrap_or_else(|_| normalize_path(allowed_root));

        // BLUE TEAM: Manually normalize the path by resolving .. components
        // This handles cases where the file doesn't exist yet (canonicalize fails)
        // First, make the path absolute by joining with the canonical root
        // if it's relative
        let absolute_resolved = if path.is_absolute() {
            resolved.clone()
        } else {
            canonical_root.join(path)
        };
        let normalized_resolved = normalize_path(&absolute_resolved);

        // Try to canonicalize the normalized path (works if file exists)
        let final_path =
            std::fs::canonicalize(&normalized_resolved).unwrap_or(normalized_resolved.clone());

        // Check if the final path is within the allowed root
        let canonical_root_str = canonical_root.to_string_lossy();
        let final_str = final_path.to_string_lossy();

        if !final_str.starts_with(&*canonical_root_str) {
            return Err(SandboxError::PathTraversal {
                path: normalized.to_string(),
            });
        }

        Ok(resolved)
    }
}

/// Normalize a path by resolving `.` and `..` components without requiring
/// the path to exist on disk.
///
/// This is the BLUE TEAM fix for path traversal detection on non-existent
/// paths. `std::fs::canonicalize` fails on non-existent paths, so we need
/// a manual normalization that resolves `..` lexically.
fn normalize_path(path: &Path) -> std::path::PathBuf {
    let mut components = Vec::new();

    for component in path.components() {
        use std::path::Component;
        match component {
            Component::CurDir => { /* skip . */ }
            Component::ParentDir => {
                // Pop the last component if it's a normal component
                // (don't pop root or prefix components)
                if let Some(last) = components.last() {
                    match last {
                        Component::Normal(_) => {
                            components.pop();
                        }
                        Component::RootDir | Component::Prefix(_) => {
                            // Can't go above root — ignore ..
                        }
                        Component::ParentDir | Component::CurDir => {
                            // Push the .. (will be resolved later or stays)
                            components.push(Component::ParentDir);
                        }
                    }
                } else {
                    components.push(Component::ParentDir);
                }
            }
            other => components.push(other),
        }
    }

    components.iter().collect()
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
