//! Minecraft mod loader profile.
//!
//! Supports two mod loaders:
//! - Fabric (modern, lightweight, 1.14+)
//! - Forge/NeoForge (traditional, heavy, 1.7+)
//!
//! Minecraft mods are JAR files with a fabric.mod.json (Fabric) or
//! mods.toml (Forge) metadata file inside.

use std::path::PathBuf;
use std::process::Command;

use super::profile::{GameProfile, InstalledMod, ModInstallResult};

/// Minecraft mod loader type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MinecraftLoader {
    Fabric,
    Forge,
    NeoForge,
}

/// Minecraft mod loader profile.
pub struct MinecraftProfile {
    loader: MinecraftLoader,
    mods_dir_override: Option<PathBuf>,
}

impl MinecraftProfile {
    pub fn new(loader: MinecraftLoader) -> Self {
        Self {
            loader,
            mods_dir_override: None,
        }
    }

    pub fn with_mods_dir(loader: MinecraftLoader, mods_dir: PathBuf) -> Self {
        Self {
            loader,
            mods_dir_override: Some(mods_dir),
        }
    }

    /// Read mod metadata from a JAR file.
    fn read_mod_info(
        jar_path: &std::path::Path,
        loader: MinecraftLoader,
    ) -> anyhow::Result<MinecraftModInfo> {
        // Use unzip to extract the metadata file
        let metadata_path = match loader {
            MinecraftLoader::Fabric => "fabric.mod.json",
            MinecraftLoader::Forge | MinecraftLoader::NeoForge => "META-INF/mods.toml",
        };

        let output = Command::new("unzip")
            .arg("-p")
            .arg(jar_path)
            .arg(metadata_path)
            .output()
            .map_err(|e| anyhow::anyhow!("failed to run unzip: {e}"))?;

        if !output.status.success() {
            anyhow::bail!("could not read {metadata_path} from {}", jar_path.display());
        }

        let content = String::from_utf8_lossy(&output.stdout);

        match loader {
            MinecraftLoader::Fabric => {
                let info: FabricModJson = serde_json::from_str(&content)
                    .map_err(|e| anyhow::anyhow!("failed to parse fabric.mod.json: {e}"))?;
                Ok(MinecraftModInfo {
                    id: info.id,
                    name: info.name,
                    version: info.version,
                })
            }
            MinecraftLoader::Forge | MinecraftLoader::NeoForge => {
                // Parse TOML mods.toml — extract modId and version
                let parsed: toml::Value = toml::from_str(&content)
                    .map_err(|e| anyhow::anyhow!("failed to parse mods.toml: {e}"))?;
                let mods = parsed
                    .get("mods")
                    .and_then(|m| m.as_array())
                    .and_then(|a| a.first())
                    .ok_or_else(|| anyhow::anyhow!("no mods entry in mods.toml"))?;
                let id = mods
                    .get("modId")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                let version = mods
                    .get("version")
                    .and_then(|v| v.as_str())
                    .unwrap_or("0.0.0")
                    .to_string();
                let name = mods
                    .get("displayName")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&id)
                    .to_string();
                Ok(MinecraftModInfo { id, name, version })
            }
        }
    }
}

impl GameProfile for MinecraftProfile {
    fn game_id(&self) -> &str {
        "minecraft"
    }

    fn game_name(&self) -> &str {
        "Minecraft"
    }

    fn detect_install_path(&self) -> Option<PathBuf> {
        if let Some(ref dir) = self.mods_dir_override {
            if dir.exists() {
                return Some(dir.clone());
            }
        }

        // Common Minecraft mods directories
        let candidates = [
            // Linux
            "~/.minecraft/mods",
            // Flatpak Linux
            "~/.var/app/com.mojang.Minecraft/.minecraft/mods",
            // Windows
            "%APPDATA%\\.minecraft\\mods",
            // macOS
            "~/Library/Application Support/minecraft/mods",
        ];

        for candidate in &candidates {
            let expanded = expand_path(candidate);
            if expanded.exists() {
                return Some(expanded);
            }
        }

        None
    }

    fn mods_directory(&self, install_path: &std::path::Path) -> PathBuf {
        install_path.to_path_buf()
    }

    fn install_mod(
        &self,
        mod_path: &std::path::Path,
        install_path: &std::path::Path,
    ) -> anyhow::Result<ModInstallResult> {
        let mods_dir = self.mods_directory(install_path);
        std::fs::create_dir_all(&mods_dir)?;

        if !mod_path.is_file() {
            anyhow::bail!("Minecraft mods must be JAR files: {}", mod_path.display());
        }

        // Read mod info
        let info = Self::read_mod_info(mod_path, self.loader)?;

        // Copy the JAR to the mods directory
        let dest_name = mod_path
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("invalid mod filename"))?
            .to_string_lossy()
            .to_string();
        let dest_path = mods_dir.join(&dest_name);
        std::fs::copy(mod_path, &dest_path)?;

        tracing::info!(
            "Installed Minecraft mod: {} v{} → {}",
            info.name,
            info.version,
            dest_path.display()
        );

        Ok(ModInstallResult {
            mod_id: info.id,
            mod_name: info.name,
            mod_version: info.version,
            installed_path: dest_path,
        })
    }

    fn list_mods(&self, install_path: &std::path::Path) -> anyhow::Result<Vec<InstalledMod>> {
        let mods_dir = self.mods_directory(install_path);
        let mut mods = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&mods_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("jar") {
                    // Try to read mod info
                    if let Ok(info) = Self::read_mod_info(&path, self.loader) {
                        mods.push(InstalledMod {
                            id: info.id,
                            name: info.name,
                            version: info.version,
                            enabled: true, // Minecraft doesn't have a mod-list.json equivalent
                            path,
                        });
                    }
                }
            }
        }

        Ok(mods)
    }

    fn enable_mod(&self, mod_id: &str, install_path: &std::path::Path) -> anyhow::Result<()> {
        // Minecraft mods are enabled by default (presence = enabled)
        // To disable, you move the .jar to .jar.disabled
        let mods_dir = self.mods_directory(install_path);

        // Check if there's a disabled version to re-enable
        if let Ok(entries) = std::fs::read_dir(&mods_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if ext == "disabled" {
                        if let Ok(info) = Self::read_mod_info(&path, self.loader) {
                            if info.id == mod_id {
                                let new_path = path.with_extension("jar");
                                std::fs::rename(&path, &new_path)?;
                                tracing::info!("Enabled Minecraft mod: {mod_id}");
                                return Ok(());
                            }
                        }
                    }
                }
            }
        }

        tracing::warn!("Mod {mod_id} not found or already enabled");
        Ok(())
    }

    fn disable_mod(&self, mod_id: &str, install_path: &std::path::Path) -> anyhow::Result<()> {
        let mods_dir = self.mods_directory(install_path);

        if let Ok(entries) = std::fs::read_dir(&mods_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("jar") {
                    if let Ok(info) = Self::read_mod_info(&path, self.loader) {
                        if info.id == mod_id {
                            let disabled_path = path.with_extension("jar.disabled");
                            std::fs::rename(&path, &disabled_path)?;
                            tracing::info!("Disabled Minecraft mod: {mod_id}");
                            return Ok(());
                        }
                    }
                }
            }
        }

        tracing::warn!("Mod {mod_id} not found");
        Ok(())
    }

    fn remove_mod(&self, mod_id: &str, install_path: &std::path::Path) -> anyhow::Result<()> {
        let mods_dir = self.mods_directory(install_path);

        if let Ok(entries) = std::fs::read_dir(&mods_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if ext == "jar" || ext == "disabled" {
                        let full_ext = if ext == "disabled" {
                            "jar.disabled"
                        } else {
                            "jar"
                        };
                        let _ = full_ext; // suppress warning
                        if let Ok(info) = Self::read_mod_info(&path, self.loader) {
                            if info.id == mod_id {
                                std::fs::remove_file(&path)?;
                                tracing::info!("Removed Minecraft mod: {mod_id}");
                                return Ok(());
                            }
                        }
                    }
                }
            }
        }

        tracing::warn!("Mod {mod_id} not found");
        Ok(())
    }
}

/// Parsed mod info.
struct MinecraftModInfo {
    id: String,
    name: String,
    version: String,
}

/// fabric.mod.json structure (partial).
#[derive(Debug, serde::Deserialize)]
struct FabricModJson {
    id: String,
    name: String,
    version: String,
}

/// Expand ~ and $HOME in a path string.
fn expand_path(path: &str) -> PathBuf {
    let expanded = if let Some(rest) = path.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            format!("{home}/{rest}")
        } else {
            path.to_string()
        }
    } else if let Some(rest) = path
        .strip_prefix("%APPDATA%\\")
        .or_else(|| path.strip_prefix("%APPDATA%/"))
    {
        if let Ok(appdata) = std::env::var("APPDATA") {
            format!("{appdata}\\{rest}")
        } else {
            path.to_string()
        }
    } else {
        path.to_string()
    };
    PathBuf::from(expanded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_id() {
        let profile = MinecraftProfile::new(MinecraftLoader::Fabric);
        assert_eq!(profile.game_id(), "minecraft");
        assert_eq!(profile.game_name(), "Minecraft");
    }

    #[test]
    fn test_loader_type() {
        let fabric = MinecraftProfile::new(MinecraftLoader::Fabric);
        let forge = MinecraftProfile::new(MinecraftLoader::Forge);
        let neo = MinecraftProfile::new(MinecraftLoader::NeoForge);
        assert_eq!(fabric.loader, MinecraftLoader::Fabric);
        assert_eq!(forge.loader, MinecraftLoader::Forge);
        assert_eq!(neo.loader, MinecraftLoader::NeoForge);
    }
}
