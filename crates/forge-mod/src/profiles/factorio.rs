//! Factorio mod loader profile.
//!
//! Factorio has a clean mod system:
//! - Mods live in `~/.factorio/mods/` (Linux) or `%APPDATA%\Factorio\mods\` (Windows)
//! - Each mod is a ZIP file named `<mod_name>_<version>.zip`
//! - Each mod contains an `info.json` file with metadata
//! - A `mod-list.json` file in the mods directory tracks which mods are enabled
//! - Mods are Lua-based (matches Forge2D scripting choice)

use std::path::PathBuf;
use std::process::Command;

use super::profile::{GameProfile, InstalledMod, ModInstallResult};

/// Factorio mod loader profile.
pub struct FactorioProfile {
    /// Override for the mods directory (if auto-detection fails)
    mods_dir_override: Option<PathBuf>,
}

impl FactorioProfile {
    pub fn new() -> Self {
        Self {
            mods_dir_override: None,
        }
    }

    /// Create with an explicit mods directory.
    pub fn with_mods_dir(mods_dir: PathBuf) -> Self {
        Self {
            mods_dir_override: Some(mods_dir),
        }
    }

    /// Read a mod's info.json from a zip file.
    fn read_mod_info(zip_path: &std::path::Path) -> anyhow::Result<FactorioModInfo> {
        // Use unzip -p to extract info.json to stdout
        let output = Command::new("unzip")
            .arg("-p")
            .arg(zip_path)
            .arg("*/info.json")
            .output()
            .map_err(|e| anyhow::anyhow!("failed to run unzip: {e}"))?;

        if !output.status.success() {
            // Try without the wildcard prefix
            let output2 = Command::new("unzip")
                .arg("-p")
                .arg(zip_path)
                .arg("info.json")
                .output()
                .map_err(|e| anyhow::anyhow!("failed to run unzip: {e}"))?;

            if !output2.status.success() {
                anyhow::bail!("could not read info.json from {}", zip_path.display());
            }

            let info: FactorioModInfo = serde_json::from_slice(&output2.stdout)?;
            return Ok(info);
        }

        let info: FactorioModInfo = serde_json::from_slice(&output.stdout)?;
        Ok(info)
    }

    /// Read the mod-list.json file.
    fn read_mod_list(mods_dir: &std::path::Path) -> anyhow::Result<FactorioModList> {
        let mod_list_path = mods_dir.join("mod-list.json");
        if !mod_list_path.exists() {
            return Ok(FactorioModList { mods: Vec::new() });
        }
        let contents = std::fs::read_to_string(&mod_list_path)?;
        let list: FactorioModList = serde_json::from_str(&contents)?;
        Ok(list)
    }

    /// Write the mod-list.json file.
    fn write_mod_list(mods_dir: &std::path::Path, list: &FactorioModList) -> anyhow::Result<()> {
        let mod_list_path = mods_dir.join("mod-list.json");
        let contents = serde_json::to_string_pretty(list)?;
        std::fs::write(&mod_list_path, contents)?;
        Ok(())
    }
}

impl Default for FactorioProfile {
    fn default() -> Self {
        Self::new()
    }
}

impl GameProfile for FactorioProfile {
    fn game_id(&self) -> &str {
        "factorio"
    }

    fn game_name(&self) -> &str {
        "Factorio"
    }

    fn detect_install_path(&self) -> Option<PathBuf> {
        if let Some(ref dir) = self.mods_dir_override {
            if dir.exists() {
                return Some(dir.clone());
            }
        }

        // Common Factorio mods directories
        let candidates = [
            // Linux
            "~/.factorio/mods",
            // Steam Linux
            "~/.steam/steam/steamapps/common/Factorio",
            // Windows
            "%APPDATA%/Factorio/mods",
            "C:\\Users\\Owner\\AppData\\Roaming\\Factorio\\mods",
        ];

        for candidate in &candidates {
            let expanded = expand_path(candidate);
            if expanded.exists() {
                // If this is the game directory, the mods are in a subdirectory
                if expanded.join("mods").exists() {
                    return Some(expanded.join("mods"));
                }
                // If this is already the mods directory
                if expanded.join("mod-list.json").exists()
                    || expanded.join("base").exists()
                    || expanded.join("core").exists()
                {
                    return Some(expanded);
                }
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

        // Factorio mods are zip files
        if !mod_path.is_file() {
            anyhow::bail!("Factorio mods must be zip files: {}", mod_path.display());
        }

        // Read the mod info from the zip
        let info = Self::read_mod_info(mod_path)?;

        // Copy the zip to the mods directory with the proper name
        let dest_name = format!("{}_{}.zip", info.name, info.version);
        let dest_path = mods_dir.join(&dest_name);
        std::fs::copy(mod_path, &dest_path)?;

        // Update mod-list.json to include the new mod
        let mut mod_list = Self::read_mod_list(&mods_dir)?;
        if !mod_list.mods.iter().any(|m| m.name == info.name) {
            mod_list.mods.push(FactorioModEntry {
                name: info.name.clone(),
                enabled: true,
            });
            Self::write_mod_list(&mods_dir, &mod_list)?;
        }

        tracing::info!(
            "Installed Factorio mod: {} v{} → {}",
            info.name,
            info.version,
            dest_path.display()
        );

        Ok(ModInstallResult {
            mod_id: info.name,
            mod_name: info.title.unwrap_or_else(|| "Unknown".to_string()),
            mod_version: info.version,
            installed_path: dest_path,
        })
    }

    fn list_mods(&self, install_path: &std::path::Path) -> anyhow::Result<Vec<InstalledMod>> {
        let mods_dir = self.mods_directory(install_path);
        let mod_list = Self::read_mod_list(&mods_dir)?;

        let mut mods = Vec::new();

        // List zip files in the mods directory
        if let Ok(entries) = std::fs::read_dir(&mods_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("zip") {
                    // Parse mod name and version from filename: <name>_<version>.zip
                    if let Some(filename) = path.file_stem().and_then(|n| n.to_str()) {
                        if let Some((name, version)) = parse_factorio_mod_filename(filename) {
                            let enabled = mod_list
                                .mods
                                .iter()
                                .find(|m| m.name == name)
                                .map(|m| m.enabled)
                                .unwrap_or(false);

                            mods.push(InstalledMod {
                                id: name.clone(),
                                name, // Would need to read info.json for the title
                                version,
                                enabled,
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
        let mut mod_list = Self::read_mod_list(&mods_dir)?;

        if let Some(mod_entry) = mod_list.mods.iter_mut().find(|m| m.name == mod_id) {
            mod_entry.enabled = true;
        } else {
            mod_list.mods.push(FactorioModEntry {
                name: mod_id.to_string(),
                enabled: true,
            });
        }

        Self::write_mod_list(&mods_dir, &mod_list)?;
        tracing::info!("Enabled Factorio mod: {mod_id}");
        Ok(())
    }

    fn disable_mod(&self, mod_id: &str, install_path: &std::path::Path) -> anyhow::Result<()> {
        let mods_dir = self.mods_directory(install_path);
        let mut mod_list = Self::read_mod_list(&mods_dir)?;

        if let Some(mod_entry) = mod_list.mods.iter_mut().find(|m| m.name == mod_id) {
            mod_entry.enabled = false;
        }

        Self::write_mod_list(&mods_dir, &mod_list)?;
        tracing::info!("Disabled Factorio mod: {mod_id}");
        Ok(())
    }

    fn remove_mod(&self, mod_id: &str, install_path: &std::path::Path) -> anyhow::Result<()> {
        let mods_dir = self.mods_directory(install_path);

        // Remove the zip file(s) matching this mod ID
        if let Ok(entries) = std::fs::read_dir(&mods_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("zip") {
                    if let Some(filename) = path.file_stem().and_then(|n| n.to_str()) {
                        if let Some((name, _)) = parse_factorio_mod_filename(filename) {
                            if name == mod_id {
                                std::fs::remove_file(&path)?;
                                tracing::info!("Removed mod file: {}", path.display());
                            }
                        }
                    }
                }
            }
        }

        // Remove from mod-list.json
        let mut mod_list = Self::read_mod_list(&mods_dir)?;
        mod_list.mods.retain(|m| m.name != mod_id);
        Self::write_mod_list(&mods_dir, &mod_list)?;

        tracing::info!("Removed Factorio mod: {mod_id}");
        Ok(())
    }
}

/// Factorio mod info.json structure.
#[derive(Debug, serde::Deserialize)]
struct FactorioModInfo {
    name: String,
    version: String,
    title: Option<String>,
}

/// Factorio mod-list.json structure.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct FactorioModList {
    #[serde(default)]
    mods: Vec<FactorioModEntry>,
}

/// A single entry in mod-list.json.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct FactorioModEntry {
    name: String,
    enabled: bool,
}

/// Parse a Factorio mod filename: "modname_1.2.3" → ("modname", "1.2.3")
fn parse_factorio_mod_filename(filename: &str) -> Option<(String, String)> {
    // Find the last underscore followed by a version-like string
    let parts: Vec<&str> = filename.rsplitn(2, '_').collect();
    if parts.len() == 2 {
        let version = parts[0];
        let name = parts[1];
        // Basic version validation
        if version.chars().all(|c| c.is_numeric() || c == '.') {
            return Some((name.to_string(), version.to_string()));
        }
    }
    None
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
        .strip_prefix("%APPDATA%/")
        .or_else(|| path.strip_prefix("%APPDATA%\\"))
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
    fn test_parse_mod_filename() {
        assert_eq!(
            parse_factorio_mod_filename("boblogistics_1.2.3"),
            Some(("boblogistics".to_string(), "1.2.3".to_string()))
        );
        assert_eq!(
            parse_factorio_mod_filename("krastorio2_2.0.1"),
            Some(("krastorio2".to_string(), "2.0.1".to_string()))
        );
        assert_eq!(parse_factorio_mod_filename("no_version"), None);
    }

    #[test]
    fn test_mod_list_serialization() {
        let list = FactorioModList {
            mods: vec![
                FactorioModEntry {
                    name: "base".to_string(),
                    enabled: true,
                },
                FactorioModEntry {
                    name: "boblogistics".to_string(),
                    enabled: false,
                },
            ],
        };
        let json = serde_json::to_string(&list).unwrap();
        let parsed: FactorioModList = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.mods.len(), 2);
        assert!(parsed.mods[0].enabled);
        assert!(!parsed.mods[1].enabled);
    }

    #[test]
    fn test_game_id() {
        let profile = FactorioProfile::new();
        assert_eq!(profile.game_id(), "factorio");
        assert_eq!(profile.game_name(), "Factorio");
    }
}
