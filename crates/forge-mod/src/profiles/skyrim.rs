//! Skyrim Anniversary Edition mod loader profile.
//!
//! Skyrim mods use a variety of formats:
//! - ESP/ESM/ESL plugins (game data modifications)
//! - BSA archives (packed assets)
//! - Loose files (unpacked assets)
//! - SKSE plugins (DLL-based script extender mods)
//!
//! Mod management typically uses a mod manager (MO2, Vortex) that
//! uses virtual file systems. This profile provides basic install/
//! list/enable/disable without a VFS layer.

use std::path::PathBuf;

use super::profile::{GameProfile, InstalledMod, ModInstallResult};

/// Skyrim AE mod loader profile.
pub struct SkyrimProfile {
    mods_dir_override: Option<PathBuf>,
}

impl SkyrimProfile {
    pub fn new() -> Self {
        Self {
            mods_dir_override: None,
        }
    }

    pub fn with_mods_dir(mods_dir: PathBuf) -> Self {
        Self {
            mods_dir_override: Some(mods_dir),
        }
    }
}

impl Default for SkyrimProfile {
    fn default() -> Self {
        Self::new()
    }
}

impl GameProfile for SkyrimProfile {
    fn game_id(&self) -> &str {
        "skyrim-ae"
    }

    fn game_name(&self) -> &str {
        "Skyrim Anniversary Edition"
    }

    fn detect_install_path(&self) -> Option<PathBuf> {
        if let Some(ref dir) = self.mods_dir_override {
            if dir.exists() {
                return Some(dir.clone());
            }
        }

        // Common Skyrim installation paths
        let candidates = [
            // Steam Linux (Proton)
            "~/.steam/steam/steamapps/common/Skyrim Special Edition",
            // Steam Windows
            "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Skyrim Special Edition",
            "C:\\Program Files\\Steam\\steamapps\\common\\Skyrim Special Edition",
            // GOG Galaxy
            "C:\\GOG Games\\Skyrim Anniversary Edition",
        ];

        for candidate in &candidates {
            let expanded = expand_path(candidate);
            if expanded.exists() {
                // The Data directory is where mods go
                let data_dir = expanded.join("Data");
                if data_dir.exists() {
                    return Some(data_dir);
                }
                return Some(expanded);
            }
        }

        None
    }

    fn mods_directory(&self, install_path: &std::path::Path) -> PathBuf {
        // If install_path is the game root, mods go in Data/
        if install_path.file_name().and_then(|n| n.to_str()) != Some("Data") {
            install_path.join("Data")
        } else {
            install_path.to_path_buf()
        }
    }

    fn install_mod(
        &self,
        mod_path: &std::path::Path,
        install_path: &std::path::Path,
    ) -> anyhow::Result<ModInstallResult> {
        let mods_dir = self.mods_directory(install_path);
        std::fs::create_dir_all(&mods_dir)?;

        let mod_id = mod_path
            .file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        if mod_path.is_file() {
            // Single file mod (ESP, BSA, etc.)
            let dest_path = mods_dir.join(mod_path.file_name().unwrap_or_default());
            std::fs::copy(mod_path, &dest_path)?;

            tracing::info!("Installed Skyrim mod file: {}", dest_path.display());

            Ok(ModInstallResult {
                mod_id: mod_id.clone(),
                mod_name: mod_id,
                mod_version: "1.0.0".to_string(),
                installed_path: dest_path,
            })
        } else if mod_path.is_dir() {
            // Directory mod — copy entire directory
            let dest_path = mods_dir.join(&mod_id);
            copy_dir_recursive(mod_path, &dest_path)?;

            tracing::info!("Installed Skyrim mod directory: {}", dest_path.display());

            Ok(ModInstallResult {
                mod_id: mod_id.clone(),
                mod_name: mod_id,
                mod_version: "1.0.0".to_string(),
                installed_path: dest_path,
            })
        } else {
            anyhow::bail!(
                "mod path is neither file nor directory: {}",
                mod_path.display()
            )
        }
    }

    fn list_mods(&self, install_path: &std::path::Path) -> anyhow::Result<Vec<InstalledMod>> {
        let mods_dir = self.mods_directory(install_path);
        let mut mods = Vec::new();

        if !mods_dir.exists() {
            return Ok(mods);
        }

        // List ESP/ESM/ESL files
        if let Ok(entries) = std::fs::read_dir(&mods_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        let ext_lower = ext.to_lowercase();
                        if matches!(ext_lower.as_str(), "esp" | "esm" | "esl") {
                            let name = path
                                .file_stem()
                                .and_then(|n| n.to_str())
                                .unwrap_or("unknown")
                                .to_string();
                            mods.push(InstalledMod {
                                id: name.clone(),
                                name,
                                version: "1.0.0".to_string(),
                                enabled: true, // presence = enabled
                                path,
                            });
                        }
                    }
                }
            }
        }

        Ok(mods)
    }

    fn enable_mod(&self, mod_id: &str, install_path: &std::path::Path) -> anyhow::Result<()> {
        let mods_dir = self.mods_directory(install_path);

        // Check for .disabled files to re-enable
        if let Ok(entries) = std::fs::read_dir(&mods_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if ext == "disabled" {
                        let stem = path.file_stem().and_then(|n| n.to_str()).unwrap_or("");
                        // The stem includes the original extension (e.g. "mod.esp")
                        if let Some(real_name) = stem.rsplit_once('.') {
                            if real_name.0 == mod_id {
                                let new_path = path.with_extension(real_name.1);
                                std::fs::rename(&path, &new_path)?;
                                tracing::info!("Enabled Skyrim mod: {mod_id}");
                                return Ok(());
                            }
                        }
                    }
                }
            }
        }

        tracing::warn!("Skyrim mod {mod_id} not found or already enabled");
        Ok(())
    }

    fn disable_mod(&self, mod_id: &str, install_path: &std::path::Path) -> anyhow::Result<()> {
        let mods_dir = self.mods_directory(install_path);

        if let Ok(entries) = std::fs::read_dir(&mods_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        let ext_lower = ext.to_lowercase();
                        if matches!(ext_lower.as_str(), "esp" | "esm" | "esl") {
                            let stem = path.file_stem().and_then(|n| n.to_str()).unwrap_or("");
                            if stem == mod_id {
                                let disabled_path = path.with_extension(format!("{ext}.disabled"));
                                std::fs::rename(&path, &disabled_path)?;
                                tracing::info!("Disabled Skyrim mod: {mod_id}");
                                return Ok(());
                            }
                        }
                    }
                }
            }
        }

        tracing::warn!("Skyrim mod {mod_id} not found");
        Ok(())
    }

    fn remove_mod(&self, mod_id: &str, install_path: &std::path::Path) -> anyhow::Result<()> {
        let mods_dir = self.mods_directory(install_path);

        if let Ok(entries) = std::fs::read_dir(&mods_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let stem = path.file_stem().and_then(|n| n.to_str()).unwrap_or("");
                    if stem == mod_id {
                        std::fs::remove_file(&path)?;
                        tracing::info!("Removed Skyrim mod: {mod_id}");
                        return Ok(());
                    }
                }
            }
        }

        tracing::warn!("Skyrim mod {mod_id} not found");
        Ok(())
    }
}

/// Copy a directory recursively.
fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> anyhow::Result<()> {
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

/// Expand ~ and $HOME in a path string.
fn expand_path(path: &str) -> PathBuf {
    let expanded = if let Some(rest) = path.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            format!("{home}/{rest}")
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
        let profile = SkyrimProfile::new();
        assert_eq!(profile.game_id(), "skyrim-ae");
        assert_eq!(profile.game_name(), "Skyrim Anniversary Edition");
    }
}
