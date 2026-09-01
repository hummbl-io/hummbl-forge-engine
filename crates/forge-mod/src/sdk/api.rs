//! Mod API — the interface that mods use to interact with the game.
//!
//! This defines the events that mods can handle and the context
//! they receive.

use std::path::PathBuf;

use crate::manifest::ModPermission;

/// The mod API version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModApiVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl ModApiVersion {
    pub fn current() -> Self {
        Self {
            major: 0,
            minor: 1,
            patch: 0,
        }
    }

    pub fn is_compatible(&self, other: &Self) -> bool {
        // Same major version = compatible
        self.major == other.major
    }
}

impl std::fmt::Display for ModApiVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Context provided to a mod when it loads.
#[derive(Debug, Clone)]
pub struct ModContext {
    /// The mod's own filesystem path
    pub mod_path: PathBuf,
    /// The game's data directory
    pub game_data_path: PathBuf,
    /// Permissions granted to this mod
    pub permissions: Vec<ModPermission>,
    /// The mod's ID
    pub mod_id: String,
    /// The mod's version
    pub mod_version: String,
}

/// Events that mods can handle.
#[derive(Debug, Clone)]
pub enum ModEvent {
    /// Called when the mod is first loaded
    OnLoad,
    /// Called when the mod is unloaded
    OnUnload,
    /// Called every game tick
    OnTick { delta_time: f64 },
    /// Called when a save is loaded
    OnSaveLoad { save_path: String },
    /// Called when the game is saving
    OnSave { save_path: String },
    /// Called when a custom event is fired
    Custom {
        name: String,
        data: serde_json::Value,
    },
}

/// Result of a mod operation.
#[derive(Debug, Clone)]
pub struct ModResult {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

impl ModResult {
    pub fn ok() -> Self {
        Self {
            success: true,
            message: String::new(),
            data: None,
        }
    }

    pub fn error(msg: impl Into<String>) -> Self {
        Self {
            success: false,
            message: msg.into(),
            data: None,
        }
    }

    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }
}
