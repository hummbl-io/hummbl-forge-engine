//! Generic game profile — for games without a dedicated profile.
//!
//! Provides basic mod management for any game that stores mods as
//! files in a directory. The user specifies the mods directory
//! explicitly.

use std::path::PathBuf;

use super::profile::{GameProfile, InstalledMod, ModInstallResult};

/// Generic mod loader profile.
pub struct GenericProfile {
    game_id: String,
    game_name: String,
    mods_dir: PathBuf,
}

impl GenericProfile {
    pub fn new(game_id: String, game_name: String, mods_dir: PathBuf) -> Self {
        Self {
            game_id,
            game_name,
            mods_dir,
        }
    }
}

impl GameProfile for GenericProfile {
    fn game_id(&self) -> &str {
        &self.game_id
    }

    fn game_name(&self) -> &str {
        &self.game_name
    }

    fn detect_install_path(&self) -> Option<PathBuf> {
        if self.mods_dir.exists() {
            Some(self.mods_dir.clone())
        } else {
            None
        }
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

        let mod_id = mod_path
            .file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        if mod_path.is_file() {
            let dest_path = mods_dir.join(mod_path.file_name().unwrap_or_default());
            std::fs::copy(mod_path, &dest_path)?;
            Ok(ModInstallResult {
                mod_id: mod_id.clone(),
                mod_name: mod_id,
                mod_version: "1.0.0".to_string(),
                installed_path: dest_path,
            })
        } else if mod_path.is_dir() {
            let dest_path = mods_dir.join(&mod_id);
            copy_dir_recursive(mod_path, &dest_path)?;
            Ok(ModInstallResult {
                mod_id: mod_id.clone(),
                mod_name: mod_id,
                mod_version: "1.0.0".to_string(),
                installed_path: dest_path,
            })
        } else {
            anyhow::bail!("mod path does not exist: {}", mod_path.display())
        }
    }

    fn list_mods(&self, install_path: &std::path::Path) -> anyhow::Result<Vec<InstalledMod>> {
        let mods_dir = self.mods_directory(install_path);
        let mut mods = Vec::new();

        if !mods_dir.exists() {
            return Ok(mods);
        }

        if let Ok(entries) = std::fs::read_dir(&mods_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = path
                    .file_stem()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                mods.push(InstalledMod {
                    id: name.clone(),
                    name,
                    version: "1.0.0".to_string(),
                    enabled: true,
                    path,
                });
            }
        }

        Ok(mods)
    }

    fn enable_mod(&self, mod_id: &str, install_path: &std::path::Path) -> anyhow::Result<()> {
        let mods_dir = self.mods_directory(install_path);

        // Check for .disabled files
        if let Ok(entries) = std::fs::read_dir(&mods_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if ext == "disabled" {
                        let stem = path.file_stem().and_then(|n| n.to_str()).unwrap_or("");
                        if stem == mod_id {
                            let new_path = path.with_extension("");
                            std::fs::rename(&path, &new_path)?;
                            tracing::info!("Enabled mod: {mod_id}");
                            return Ok(());
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn disable_mod(&self, mod_id: &str, install_path: &std::path::Path) -> anyhow::Result<()> {
        let mods_dir = self.mods_directory(install_path);

        if let Ok(entries) = std::fs::read_dir(&mods_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let stem = path.file_stem().and_then(|n| n.to_str()).unwrap_or("");
                    if stem == mod_id {
                        let disabled_path = path.with_extension("disabled");
                        std::fs::rename(&path, &disabled_path)?;
                        tracing::info!("Disabled mod: {mod_id}");
                        return Ok(());
                    }
                }
            }
        }

        Ok(())
    }

    fn remove_mod(&self, mod_id: &str, install_path: &std::path::Path) -> anyhow::Result<()> {
        let mods_dir = self.mods_directory(install_path);

        if let Ok(entries) = std::fs::read_dir(&mods_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let stem = path.file_stem().and_then(|n| n.to_str()).unwrap_or("");
                if stem == mod_id {
                    if path.is_dir() {
                        std::fs::remove_dir_all(&path)?;
                    } else {
                        std::fs::remove_file(&path)?;
                    }
                    tracing::info!("Removed mod: {mod_id}");
                    return Ok(());
                }
            }
        }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generic_profile() {
        let profile = GenericProfile::new(
            "my-game".to_string(),
            "My Game".to_string(),
            PathBuf::from("/tmp/mods"),
        );
        assert_eq!(profile.game_id(), "my-game");
        assert_eq!(profile.game_name(), "My Game");
    }
}
