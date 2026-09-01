//! Lockfile generation and parsing.
//!
//! `forge.lock` pins exact versions of engine, adapter, game, and mods —
//! like Cargo.lock but for a Forge Engine project.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lockfile {
    /// Lockfile format version
    pub format_version: u32,
    /// Project ID
    pub project_id: String,
    /// Project version
    pub project_version: String,
    /// Engine version pin
    pub engine: EnginePin,
    /// Mod version pins
    #[serde(default)]
    pub mods: Vec<ModPin>,
    /// Asset hashes
    #[serde(default)]
    pub assets: Vec<AssetPin>,
    /// When the lockfile was generated
    pub generated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnginePin {
    pub kind: String,
    pub version: String,
    pub adapter_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModPin {
    pub id: String,
    pub version: String,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetPin {
    pub path: String,
    pub hash: String,
}
