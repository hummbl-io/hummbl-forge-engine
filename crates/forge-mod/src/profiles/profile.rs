//! Game profile trait — the interface every universal mod loader profile implements.

use std::path::PathBuf;

/// A game profile knows how to manage mods for a specific game.
pub trait GameProfile: Send + Sync {
    /// The game identifier (e.g. "factorio", "skyrim", "minecraft")
    fn game_id(&self) -> &str;

    /// Human-readable game name
    fn game_name(&self) -> &str;

    /// Try to auto-detect the game installation path.
    fn detect_install_path(&self) -> Option<PathBuf>;

    /// Get the mods directory for this game.
    /// Returns None if the game is not installed or the mods dir doesn't exist.
    fn mods_directory(&self, install_path: &std::path::Path) -> PathBuf;

    /// Install a mod from a path (archive or directory).
    fn install_mod(
        &self,
        mod_path: &std::path::Path,
        install_path: &std::path::Path,
    ) -> anyhow::Result<ModInstallResult>;

    /// List installed mods.
    fn list_mods(&self, install_path: &std::path::Path) -> anyhow::Result<Vec<InstalledMod>>;

    /// Enable a mod by ID.
    fn enable_mod(&self, mod_id: &str, install_path: &std::path::Path) -> anyhow::Result<()>;

    /// Disable a mod by ID.
    fn disable_mod(&self, mod_id: &str, install_path: &std::path::Path) -> anyhow::Result<()>;

    /// Remove a mod by ID.
    fn remove_mod(&self, mod_id: &str, install_path: &std::path::Path) -> anyhow::Result<()>;
}

/// Result of installing a mod.
#[derive(Debug)]
pub struct ModInstallResult {
    pub mod_id: String,
    pub mod_name: String,
    pub mod_version: String,
    pub installed_path: PathBuf,
}

/// A mod that is installed for a game.
#[derive(Debug, Clone, serde::Serialize)]
pub struct InstalledMod {
    pub id: String,
    pub name: String,
    pub version: String,
    pub enabled: bool,
    pub path: PathBuf,
}
