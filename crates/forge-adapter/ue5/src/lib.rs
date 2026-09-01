//! UE5 adapter — communicates with Unreal Engine 5 via UAT/UBT subprocess.
//!
//! Uses Unreal Automation Tool (UAT) and UnrealBuildTool (UBT) to build
//! UE5 projects programmatically.

use std::path::PathBuf;
use std::process::Command;

use forge_adapter::{BuildMetadata, BuildOutput, EngineAdapter};
use forge_core::config::EngineConfig;
use forge_core::target::EngineKind;
use forge_core::ForgeProject;

/// UE5 engine adapter.
pub struct Ue5Adapter {
    /// Path to UE5 installation (Engine directory)
    engine_path: PathBuf,
    /// UE5 version string (e.g. "5.4", "5.5")
    engine_version: String,
}

impl Ue5Adapter {
    /// Create a new UE5 adapter from an engine path.
    pub fn new(engine_path: impl Into<PathBuf>, version: impl Into<String>) -> Self {
        Self {
            engine_path: engine_path.into(),
            engine_version: version.into(),
        }
    }

    /// Try to auto-detect UE5 installation.
    pub fn auto_detect() -> Option<Self> {
        // Common UE5 installation paths
        let candidates: &[(&str, &str)] = &[
            ("C:\\Program Files\\Epic Games\\UE_5.5\\Engine", "5.5"),
            ("C:\\Program Files\\Epic Games\\UE_5.4\\Engine", "5.4"),
            ("C:\\Program Files\\Epic Games\\UE_5.3\\Engine", "5.3"),
            ("/home/reuben/UE_5.4/Engine", "5.4"),
            ("/opt/UE_5.4/Engine", "5.4"),
        ];

        for (path, version) in candidates {
            if PathBuf::from(path).exists() {
                return Some(Self::new(path, *version));
            }
        }

        None
    }

    /// Create from a specific engine path and version.
    pub fn from_config(engine_path: &str, version: &str) -> Self {
        Self::new(engine_path, version)
    }

    /// Get the path to the Unreal Automation Tool (UAT) script.
    fn uat_path(&self) -> PathBuf {
        // UAT is at Engine/Build/BatchFiles/RunUAT.bat (Windows)
        // or Engine/Build/BatchFiles/RunUAT.sh (Linux)
        if cfg!(windows) {
            self.engine_path.join("Build/BatchFiles/RunUAT.bat")
        } else {
            self.engine_path.join("Build/BatchFiles/RunUAT.sh")
        }
    }

    /// Map a Forge platform string to a UE5 platform name.
    fn ue5_platform(platform: &str) -> Option<&'static str> {
        match platform {
            "linux-native" | "linux-steam" => Some("Linux"),
            "windows-native" | "windows-steam" => Some("Win64"),
            "xbox" => Some("GDKLX"), // Xbox Series X/S
            _ => None,
        }
    }

    /// Map a Forge profile string to a UE5 configuration.
    fn ue5_config(profile: &str) -> &'static str {
        match profile {
            "debug" => "Debug",
            "shipping" => "Shipping",
            _ => "Development",
        }
    }

    /// Build the UAT BuildCookRun command.
    fn build_uat_command(
        &self,
        uproject_path: &std::path::Path,
        platform: &str,
        profile: &str,
        archive_dir: &std::path::Path,
    ) -> anyhow::Result<Command> {
        let uat = self.uat_path();
        if !uat.exists() {
            anyhow::bail!("UAT not found at {}", uat.display());
        }

        let ue5_platform = Self::ue5_platform(platform)
            .ok_or_else(|| anyhow::anyhow!("unsupported UE5 platform: {platform}"))?;
        let ue5_config = Self::ue5_config(profile);

        let mut cmd = Command::new(&uat);
        cmd.arg("BuildCookRun")
            .arg("-project=")
            .arg(uproject_path)
            .arg("-platform=")
            .arg(ue5_platform)
            .arg("-clientconfig=")
            .arg(ue5_config)
            .arg("-cook")
            .arg("-build")
            .arg("-stage")
            .arg("-pak")
            .arg("-archive")
            .arg("-archivedirectory=")
            .arg(archive_dir)
            .arg("-nocompileeditor")
            .arg("-installed");

        // For Steam targets, add Steam-specific flags
        if platform == "linux-steam" || platform == "windows-steam" {
            cmd.arg("-steamstage");
        }

        Ok(cmd)
    }

    /// Find the packaged binary in the archive directory after build.
    fn find_packaged_binary(
        &self,
        archive_dir: &std::path::Path,
        platform: &str,
        project_name: &str,
    ) -> Option<PathBuf> {
        // UE5 archives to: <archive_dir>/<Platform>/<ProjectName>/Binaries/<Platform>/<ProjectName>
        let ue5_platform = Self::ue5_platform(platform)?;
        let binary_dir = archive_dir
            .join(ue5_platform)
            .join(project_name)
            .join("Binaries")
            .join(ue5_platform);

        // Look for the binary
        let binary_name = if cfg!(windows) || ue5_platform == "Win64" {
            format!("{project_name}.exe")
        } else {
            project_name.to_string()
        };

        let binary_path = binary_dir.join(&binary_name);
        if binary_path.exists() {
            Some(binary_path)
        } else {
            // Try to find any executable in the directory
            if let Ok(entries) = std::fs::read_dir(&binary_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        return Some(path);
                    }
                }
            }
            None
        }
    }
}

#[async_trait::async_trait]
impl EngineAdapter for Ue5Adapter {
    fn engine_kind(&self) -> EngineKind {
        EngineKind::Ue5
    }

    async fn build(
        &self,
        project: &ForgeProject,
        platform: &str,
        profile: &str,
    ) -> anyhow::Result<BuildOutput> {
        let (uproject, version) = match &project.engine {
            EngineConfig::Ue5 {
                uproject, version, ..
            } => (uproject, version),
            _ => anyhow::bail!("project is not a UE5 project"),
        };

        tracing::info!("UE5 build: {uproject} for {platform} ({profile}) via UE5 {version}");

        // Resolve the .uproject path (relative to forge.toml directory)
        let uproject_path = std::path::Path::new(uproject);
        if !uproject_path.exists() {
            anyhow::bail!(".uproject file not found at {}", uproject_path.display());
        }

        // Create archive directory for build output
        let archive_dir = std::path::Path::new("dist/ue5-build");
        std::fs::create_dir_all(archive_dir)?;

        // Build the UAT command
        let mut cmd = self.build_uat_command(uproject_path, platform, profile, archive_dir)?;
        tracing::info!("Running UAT: {:?}", cmd);

        // Execute
        let output = cmd
            .output()
            .map_err(|e| anyhow::anyhow!("failed to run UAT: {e}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            anyhow::bail!("UAT build failed:\nstdout: {stdout}\nstderr: {stderr}")
        }

        // Find the packaged binary
        let project_name = &project.project.name;
        let binary_path = self
            .find_packaged_binary(archive_dir, platform, project_name)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "build succeeded but packaged binary not found in {}",
                    archive_dir.display()
                )
            })?;

        tracing::info!("UE5 build complete: {}", binary_path.display());

        Ok(BuildOutput {
            output_path: binary_path.clone(),
            metadata: BuildMetadata {
                engine: "ue5".to_string(),
                engine_version: self.engine_version.clone(),
                platform: platform.to_string(),
                profile: profile.to_string(),
                build_hash: compute_file_hash(&binary_path)?,
                built_at: current_timestamp(),
            },
        })
    }
}

/// Compute SHA-256 hash of a file.
fn compute_file_hash(path: &std::path::Path) -> anyhow::Result<String> {
    use sha2::{Digest, Sha256};
    let data =
        std::fs::read(path).map_err(|e| anyhow::anyhow!("failed to read file for hashing: {e}"))?;
    let mut hasher = Sha256::new();
    hasher.update(&data);
    Ok(hex::encode(hasher.finalize()))
}

fn current_timestamp() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{now}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ue5_platform_mapping() {
        assert_eq!(Ue5Adapter::ue5_platform("linux-native"), Some("Linux"));
        assert_eq!(Ue5Adapter::ue5_platform("linux-steam"), Some("Linux"));
        assert_eq!(Ue5Adapter::ue5_platform("windows-native"), Some("Win64"));
        assert_eq!(Ue5Adapter::ue5_platform("xbox"), Some("GDKLX"));
        assert_eq!(Ue5Adapter::ue5_platform("web"), None);
    }

    #[test]
    fn test_ue5_config_mapping() {
        assert_eq!(Ue5Adapter::ue5_config("debug"), "Debug");
        assert_eq!(Ue5Adapter::ue5_config("release"), "Development");
        assert_eq!(Ue5Adapter::ue5_config("shipping"), "Shipping");
    }
}
