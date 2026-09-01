//! Mod SDK — tools for mod authors.
//!
//! Provides the API surface that mods call when running inside a
//! Forge Engine game. This is the "moddable-by-design" layer.

/// The mod API version.
pub const MOD_API_VERSION: &str = "0.1.0";

/// Context provided to a mod when it loads.
#[derive(Debug)]
pub struct ModContext {
    /// The mod's own filesystem path
    pub mod_path: std::path::PathBuf,
    /// The game's data directory
    pub game_data_path: std::path::PathBuf,
    /// Permissions granted to this mod
    pub permissions: Vec<crate::manifest::ModPermission>,
}
