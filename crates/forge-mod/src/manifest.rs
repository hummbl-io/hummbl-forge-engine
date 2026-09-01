//! Mod manifest schema (mod.toml).

use serde::{Deserialize, Serialize};

/// The root configuration for a mod.
///
/// This is the `mod.toml` file that sits at the root of every mod.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModManifest {
    /// Mod metadata
    pub mod_info: ModInfo,

    /// Target game + compatible versions
    pub target: ModTarget,

    /// Mod dependencies (other mods this depends on)
    #[serde(default)]
    pub dependencies: Vec<ModDependency>,

    /// Content hooks (what this mod replaces/adds)
    #[serde(default)]
    pub hooks: Vec<ContentHook>,

    /// Permissions the mod requests
    #[serde(default)]
    pub permissions: Vec<ModPermission>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModInfo {
    /// Unique mod identifier
    pub id: String,
    /// Human-readable mod name
    pub name: String,
    /// Mod author
    pub author: String,
    /// Mod version (semver)
    pub version: String,
    /// Mod description
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModTarget {
    /// Target game ID (e.g. "com.hummbl.mygame" or "factorio")
    pub game: String,
    /// Compatible game version range (semver range)
    pub compatible_versions: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModDependency {
    /// Dependency mod ID
    pub id: String,
    /// Compatible version range
    pub version: String,
    /// Whether this dependency is optional
    #[serde(default)]
    pub optional: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentHook {
    /// Hook type (asset_replace, asset_add, script_hook, logic_override)
    pub kind: HookKind,
    /// Target path in the game (what this hook affects)
    pub target: String,
    /// Source path in the mod (what to load)
    pub source: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HookKind {
    /// Replace an existing asset
    AssetReplace,
    /// Add a new asset
    AssetAdd,
    /// Hook into a script event
    ScriptHook,
    /// Override game logic
    LogicOverride,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ModPermission {
    /// Read game files
    ReadFiles,
    /// Write game files (save data)
    WriteFiles,
    /// Network access
    Network,
    /// Execute arbitrary code
    ExecuteCode,
    /// Access player input
    PlayerInput,
    /// Access game state
    GameState,
}

impl ModManifest {
    pub fn from_file(path: &std::path::Path) -> Result<Self, toml::de::Error> {
        let contents = std::fs::read_to_string(path).unwrap_or_default();
        toml::from_str(&contents)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal_mod_toml() {
        let toml_str = r#"
[mod_info]
id = "com.example.mymod"
name = "My Mod"
author = "Reuben"
version = "1.0.0"
description = "A test mod"

[target]
game = "com.hummbl.mygame"
compatible_versions = ">=0.1.0"

[[hooks]]
kind = "asset_add"
target = "textures/"
source = "my_textures/"
"#;
        let manifest: ModManifest = toml::from_str(toml_str).unwrap();
        assert_eq!(manifest.mod_info.id, "com.example.mymod");
        assert_eq!(manifest.hooks.len(), 1);
        assert!(matches!(manifest.hooks[0].kind, HookKind::AssetAdd));
    }
}
