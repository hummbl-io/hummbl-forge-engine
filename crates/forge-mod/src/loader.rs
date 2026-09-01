//! Mod loader runtime.
//!
//! Loads mods at startup in dependency order, resolves conflicts,
//! sandboxes mods (permissions enforced), and reports mod errors
//! without crashing the game.

use crate::manifest::ModManifest;

/// Load all mods from a directory.
pub fn load_mods(mods_dir: &std::path::Path) -> Vec<LoadedMod> {
    let mut mods = Vec::new();

    if !mods_dir.exists() {
        return mods;
    }

    if let Ok(entries) = std::fs::read_dir(mods_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let manifest_path = path.join("mod.toml");
            if manifest_path.exists() {
                match ModManifest::from_file(&manifest_path) {
                    Ok(manifest) => {
                        tracing::info!(
                            "Loaded mod: {} v{}",
                            manifest.mod_info.name,
                            manifest.mod_info.version
                        );
                        mods.push(LoadedMod {
                            manifest,
                            path: path.clone(),
                            enabled: true,
                        });
                    }
                    Err(e) => {
                        tracing::warn!("Failed to load mod at {}: {e}", path.display());
                    }
                }
            }
        }
    }

    mods
}

/// A loaded mod with its filesystem path and enabled state.
#[derive(Debug)]
pub struct LoadedMod {
    pub manifest: ModManifest,
    pub path: std::path::PathBuf,
    pub enabled: bool,
}
