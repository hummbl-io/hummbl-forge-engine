//! Steam publish pipeline (SteamPipe).
//!
//! Uses the Steam Web API and SteamCMD to upload builds to Steam.
//! Requires a Steamworks SDK account and app ID configuration.

use std::path::Path;
use std::process::Command;

use crate::store::{Store, UploadReceipt};

/// Steam publish configuration.
#[derive(Debug, Clone)]
pub struct SteamConfig {
    /// Steam app ID
    pub app_id: u32,
    /// Steam depot ID
    pub depot_id: u32,
    /// Steam Web API key
    pub api_key: String,
    /// Steam account username
    pub username: String,
    /// Path to steamcmd executable
    pub steamcmd_path: String,
    /// Build description
    pub build_description: String,
    /// Whether to dry-run (don't actually upload)
    pub dry_run: bool,
}

/// Steam store plugin.
pub struct SteamStore {
    config: SteamConfig,
}

impl SteamStore {
    pub fn new(config: SteamConfig) -> Self {
        Self { config }
    }

    /// Generate a Steam VDF (Valve Data Format) build script.
    fn generate_vdf(&self, content_path: &Path) -> anyhow::Result<std::path::PathBuf> {
        let vdf_path = content_path.join("steam_build.vdf");
        let vdf_content = format!(
            r#""appbuild"
{{
    "appid" "{appid}"
    "desc" "{desc}"
    "buildpath" "build/"
    "contentroot" "{root}"
    "depots"
    {{
        "{depotid}"
        {{
            "source" "{root}"
            "executable" "game"
            "FileMapping"
            {{
                "LocalPath" "*"
                "DepotPath" "."
                "recursive" "1"
            }}
        }}
    }}
}}
"#,
            appid = self.config.app_id,
            desc = self.config.build_description,
            root = content_path.display(),
            depotid = self.config.depot_id,
        );

        std::fs::write(&vdf_path, vdf_content)?;
        Ok(vdf_path)
    }
}

#[async_trait::async_trait]
impl Store for SteamStore {
    fn target(&self) -> forge_core::PublishTarget {
        forge_core::PublishTarget::Steam
    }

    async fn package(&self, artifact_path: &Path) -> anyhow::Result<std::path::PathBuf> {
        tracing::info!(
            "Packaging for Steam: app {} depot {}",
            self.config.app_id,
            self.config.depot_id
        );

        // Create a build directory structure
        let build_dir = artifact_path
            .parent()
            .unwrap_or(Path::new("."))
            .join("steam-build");
        std::fs::create_dir_all(&build_dir)?;

        // Copy the artifact to the build directory
        if artifact_path.is_file() {
            let dest = build_dir.join("game");
            std::fs::copy(artifact_path, &dest)?;
            make_executable(&dest)?;
        } else if artifact_path.is_dir() {
            copy_dir_recursive(artifact_path, &build_dir)?;
        }

        // Generate the VDF build script
        let vdf_path = self.generate_vdf(&build_dir)?;

        tracing::info!(
            "Steam package ready: {} (VDF: {})",
            build_dir.display(),
            vdf_path.display()
        );
        Ok(build_dir)
    }

    async fn upload(&self, package_path: &Path) -> anyhow::Result<UploadReceipt> {
        if self.config.dry_run {
            tracing::info!(
                "Steam upload: DRY RUN (would upload to app {})",
                self.config.app_id
            );
            return Ok(UploadReceipt {
                store: "steam".to_string(),
                uploaded_at: current_timestamp(),
                build_hash: String::new(),
                store_build_id: Some(format!("dry-run-{}", self.config.app_id)),
                status: "dry-run".to_string(),
            });
        }

        let vdf_path = package_path.join("steam_build.vdf");
        if !vdf_path.exists() {
            anyhow::bail!("VDF build script not found: {}", vdf_path.display());
        }

        tracing::info!(
            "Uploading to Steam: app {} via {}",
            self.config.app_id,
            self.config.steamcmd_path
        );

        // Run steamcmd to upload
        let output = Command::new(&self.config.steamcmd_path)
            .arg(format!("+login {}", self.config.username))
            .arg("+run_app_build")
            .arg(&vdf_path)
            .arg("+quit")
            .output()
            .map_err(|e| anyhow::anyhow!("failed to run steamcmd: {e}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("steamcmd upload failed: {stderr}");
        }

        Ok(UploadReceipt {
            store: "steam".to_string(),
            uploaded_at: current_timestamp(),
            build_hash: String::new(),
            store_build_id: Some(format!("app-{}", self.config.app_id)),
            status: "uploaded".to_string(),
        })
    }
}

/// itch.io publish configuration.
#[derive(Debug, Clone)]
pub struct ItchConfig {
    /// itch.io project name (user/game)
    pub project: String,
    /// itch.io API key
    pub api_key: String,
    /// Path to butler executable
    pub butler_path: String,
    /// Channel name (e.g. "linux", "windows", "web")
    pub channel: String,
    /// Whether to dry-run
    pub dry_run: bool,
}

/// itch.io store plugin.
pub struct ItchStore {
    config: ItchConfig,
}

impl ItchStore {
    pub fn new(config: ItchConfig) -> Self {
        Self { config }
    }
}

#[async_trait::async_trait]
impl Store for ItchStore {
    fn target(&self) -> forge_core::PublishTarget {
        // itch.io doesn't have a dedicated PublishTarget — use Direct for now
        forge_core::PublishTarget::Direct
    }

    async fn package(&self, artifact_path: &Path) -> anyhow::Result<std::path::PathBuf> {
        tracing::info!(
            "Packaging for itch.io: {} channel {}",
            self.config.project,
            self.config.channel
        );
        // itch.io accepts the artifact as-is (butler handles packaging)
        Ok(artifact_path.to_path_buf())
    }

    async fn upload(&self, package_path: &Path) -> anyhow::Result<UploadReceipt> {
        if self.config.dry_run {
            tracing::info!(
                "itch.io upload: DRY RUN (would push to {})",
                self.config.project
            );
            return Ok(UploadReceipt {
                store: "itch.io".to_string(),
                uploaded_at: current_timestamp(),
                build_hash: String::new(),
                store_build_id: None,
                status: "dry-run".to_string(),
            });
        }

        tracing::info!("Uploading to itch.io: {} via butler", self.config.project);

        let output = Command::new(&self.config.butler_path)
            .arg("push")
            .arg(package_path)
            .arg(format!("{}/{}", self.config.project, self.config.channel))
            .env("BUTLER_API_KEY", &self.config.api_key)
            .output()
            .map_err(|e| anyhow::anyhow!("failed to run butler: {e}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("butler upload failed: {stderr}");
        }

        Ok(UploadReceipt {
            store: "itch.io".to_string(),
            uploaded_at: current_timestamp(),
            build_hash: String::new(),
            store_build_id: None,
            status: "uploaded".to_string(),
        })
    }
}

/// Xbox GDK publish pipeline.
pub struct XboxStore {
    config: XboxConfig,
}

/// Xbox GDK publish configuration.
#[derive(Debug, Clone)]
pub struct XboxConfig {
    /// Xbox partner center app ID
    pub app_id: String,
    /// Title ID
    pub title_id: String,
    /// Package identity (from partner center)
    pub package_identity: String,
    /// Path to MakePkg.exe
    pub makepkg_path: String,
    /// Whether to dry-run
    pub dry_run: bool,
}

impl XboxStore {
    pub fn new(config: XboxConfig) -> Self {
        Self { config }
    }
}

#[async_trait::async_trait]
impl Store for XboxStore {
    fn target(&self) -> forge_core::PublishTarget {
        forge_core::PublishTarget::Xbox
    }

    async fn package(&self, artifact_path: &Path) -> anyhow::Result<std::path::PathBuf> {
        tracing::info!("Packaging for Xbox GDK: {}", self.config.title_id);

        // Create an Xbox package layout
        let pkg_dir = artifact_path
            .parent()
            .unwrap_or(Path::new("."))
            .join("xbox-package");
        std::fs::create_dir_all(&pkg_dir)?;

        // Copy the artifact
        if artifact_path.is_file() {
            std::fs::copy(artifact_path, pkg_dir.join("game.exe"))?;
        } else if artifact_path.is_dir() {
            copy_dir_recursive(artifact_path, &pkg_dir)?;
        }

        // Generate MicrosoftGame.config
        let config_xml = format!(
            r#"<?xml version="1.0" encoding="utf-8"?>
<Game configVersion="1">
  <Identity Name="{name}" Publisher="CN=HUMMBL" />
  <TitleId>{title_id}</TitleId>
  <ShellExecutable>game.exe</ShellExecutable>
  <TargetDevice>PC</TargetDevice>
</Game>
"#,
            name = self.config.package_identity,
            title_id = self.config.title_id,
        );
        std::fs::write(pkg_dir.join("MicrosoftGame.config"), config_xml)?;

        tracing::info!("Xbox package ready: {}", pkg_dir.display());
        Ok(pkg_dir)
    }

    async fn upload(&self, _package_path: &Path) -> anyhow::Result<UploadReceipt> {
        if self.config.dry_run {
            tracing::info!(
                "Xbox upload: DRY RUN (would upload to {})",
                self.config.title_id
            );
            return Ok(UploadReceipt {
                store: "xbox".to_string(),
                uploaded_at: current_timestamp(),
                build_hash: String::new(),
                store_build_id: Some(format!("dry-run-{}", self.config.title_id)),
                status: "dry-run".to_string(),
            });
        }

        // In a real implementation, this would use the Xbox Partner Center API
        // or the Microsoft Store submission API
        tracing::warn!("Xbox GDK upload not yet implemented — requires Partner Center API");
        Ok(UploadReceipt {
            store: "xbox".to_string(),
            uploaded_at: current_timestamp(),
            build_hash: String::new(),
            store_build_id: None,
            status: "not-implemented".to_string(),
        })
    }
}

/// Auto-update / patching system.
pub mod auto_update {
    use std::path::Path;

    /// A delta patch between two versions.
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct DeltaPatch {
        pub from_version: String,
        pub to_version: String,
        pub patch_file: String,
        pub patch_hash: String,
        pub patch_size: u64,
        pub original_size: u64,
    }

    /// Generate a delta patch between two build artifacts.
    ///
    /// Currently uses a simple approach: if the artifacts are similar enough,
    /// produce a binary diff. In production, this would use bsdiff or similar.
    pub fn generate_patch(
        _old_artifact: &Path,
        _new_artifact: &Path,
        from_version: &str,
        to_version: &str,
    ) -> anyhow::Result<DeltaPatch> {
        tracing::info!("Generating delta patch: {from_version} → {to_version}");
        // TODO: implement bsdiff or similar binary diffing
        anyhow::bail!("delta patch generation not yet implemented — use full builds for now")
    }

    /// Check for available updates.
    pub fn check_for_updates(
        _current_version: &str,
        _update_server_url: &str,
    ) -> anyhow::Result<Vec<DeltaPatch>> {
        // TODO: query an update server for available patches
        Ok(Vec::new())
    }
}

// Helper functions

fn current_timestamp() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{now}")
}

fn make_executable(path: &Path) -> anyhow::Result<()> {
    #[cfg(not(unix))]
    let _ = path;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(path)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(path, perms)?;
    }
    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_steam_config() {
        let config = SteamConfig {
            app_id: 123456,
            depot_id: 123457,
            api_key: "test-key".to_string(),
            username: "testuser".to_string(),
            steamcmd_path: "/usr/bin/steamcmd".to_string(),
            build_description: "Test build".to_string(),
            dry_run: true,
        };
        let store = SteamStore::new(config);
        assert_eq!(store.config.app_id, 123456);
    }

    #[test]
    fn test_itch_config() {
        let config = ItchConfig {
            project: "user/game".to_string(),
            api_key: "test-key".to_string(),
            butler_path: "/usr/bin/butler".to_string(),
            channel: "linux".to_string(),
            dry_run: true,
        };
        let store = ItchStore::new(config);
        assert_eq!(store.config.project, "user/game");
    }

    #[test]
    fn test_xbox_config() {
        let config = XboxConfig {
            app_id: "test-app".to_string(),
            title_id: "12345".to_string(),
            package_identity: "HUMMBL.TestGame".to_string(),
            makepkg_path: "MakePkg.exe".to_string(),
            dry_run: true,
        };
        let store = XboxStore::new(config);
        assert_eq!(store.config.title_id, "12345");
    }
}
