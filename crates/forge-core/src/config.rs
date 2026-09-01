//! forge.toml configuration schema — the single source of truth for a game project.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The root configuration file for a Forge Engine game project.
///
/// This is the `forge.toml` file that sits at the root of every game project
/// built with Forge Engine. It declares: which engine, which platforms, which
/// mods are allowed, what version, and what assets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgeProject {
    /// Project metadata
    #[serde(default)]
    pub project: ProjectMeta,

    /// Which engine to use (ue5, forge2d)
    pub engine: EngineConfig,

    /// Build targets (platforms to build for)
    #[serde(default)]
    pub build: BuildConfig,

    /// Publish targets (stores to publish to)
    #[serde(default)]
    pub publish: PublishConfig,

    /// Mod policy for this game
    #[serde(default)]
    pub mods: ModConfig,

    /// Asset configuration
    #[serde(default)]
    pub assets: AssetConfig,

    /// Version (semver)
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectMeta {
    /// Human-readable project name
    pub name: String,

    /// Unique project identifier (reverse-DNS style, e.g. "com.hummbl.mygame")
    pub id: String,

    /// Project description
    #[serde(default)]
    pub description: String,

    /// Author(s)
    #[serde(default)]
    pub authors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum EngineConfig {
    /// Unreal Engine 5
    Ue5 {
        /// Path to the .uproject file (relative to forge.toml)
        uproject: String,

        /// UE5 engine version (e.g. "5.4", "5.5")
        version: String,

        /// Path to UE5 installation (optional — auto-detected if omitted)
        #[serde(default)]
        engine_path: Option<String>,
    },

    /// Forge2D (Bevy-based custom engine)
    Forge2d {
        /// Path to the game's Cargo.toml (relative to forge.toml)
        cargo_toml: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BuildConfig {
    /// Platforms to build for
    #[serde(default)]
    pub targets: Vec<String>,

    /// Build profile (debug, release, shipping)
    #[serde(default = "default_profile")]
    pub profile: String,

    /// Additional build arguments passed to the engine
    #[serde(default)]
    pub extra_args: Vec<String>,
}

fn default_profile() -> String {
    "release".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PublishConfig {
    /// Stores to publish to
    #[serde(default)]
    pub targets: Vec<String>,

    /// Whether to use dry-run mode (build package without uploading)
    #[serde(default = "default_true")]
    pub dry_run: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModConfig {
    /// Whether mods are enabled for this game
    #[serde(default)]
    pub enabled: bool,

    /// Mod API version
    #[serde(default)]
    pub api_version: Option<String>,

    /// Directory where mods are loaded from
    #[serde(default = "default_mods_dir")]
    pub mods_dir: String,

    /// Whether mod sandboxing is enforced
    #[serde(default = "default_true")]
    pub sandbox: bool,
}

fn default_mods_dir() -> String {
    "mods".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AssetConfig {
    /// Directory containing game assets
    #[serde(default = "default_assets_dir")]
    pub assets_dir: String,

    /// Blender pipeline configuration
    #[serde(default)]
    pub blender: BlenderConfig,
}

fn default_assets_dir() -> String {
    "assets".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BlenderConfig {
    /// Path to Blender executable (auto-detected if omitted)
    #[serde(default)]
    pub path: Option<String>,

    /// Directory containing .blend source files
    #[serde(default = "default_blender_src")]
    pub source_dir: String,

    /// Export format (glTF, FBX)
    #[serde(default = "default_export_format")]
    pub export_format: String,
}

fn default_blender_src() -> String {
    "blender".to_string()
}

fn default_export_format() -> String {
    "glTF".to_string()
}

impl ForgeProject {
    /// Load a forge.toml from a file path.
    pub fn from_file(path: &std::path::Path) -> Result<ForgeProject, ForgeProjectError> {
        let contents = std::fs::read_to_string(path)
            .map_err(|e| ForgeProjectError::ReadFailed(path.display().to_string(), e))?;
        let project: ForgeProject =
            toml::from_str(&contents).map_err(ForgeProjectError::ParseFailed)?;
        Ok(project)
    }

    /// Load a forge.toml from the current directory.
    pub fn from_current_dir() -> Result<ForgeProject, ForgeProjectError> {
        Self::from_file(std::path::Path::new("forge.toml"))
    }

    /// Validate the project configuration.
    pub fn validate(&self) -> Result<(), ForgeProjectError> {
        if self.project.name.is_empty() {
            return Err(ForgeProjectError::Invalid("project.name is empty".into()));
        }
        if self.project.id.is_empty() {
            return Err(ForgeProjectError::Invalid("project.id is empty".into()));
        }
        if self.version.is_empty() {
            return Err(ForgeProjectError::Invalid("version is empty".into()));
        }
        // Validate semver
        semver::Version::parse(&self.version)
            .map_err(|e| ForgeProjectError::Invalid(format!("version is not valid semver: {e}")))?;
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum ForgeProjectError {
    #[error("failed to read forge.toml at {0}: {1}")]
    ReadFailed(String, std::io::Error),

    #[error("failed to parse forge.toml: {0}")]
    ParseFailed(#[from] toml::de::Error),

    #[error("invalid project configuration: {0}")]
    Invalid(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal_forge_toml() {
        let toml_str = r#"
version = "0.1.0"

[project]
name = "Test Game"
id = "com.hummbl.testgame"

[engine]
kind = "forge2d"
cargo_toml = "Cargo.toml"

[build]
targets = ["linux-native"]
"#;
        let project: ForgeProject = toml::from_str(toml_str).unwrap();
        assert_eq!(project.project.name, "Test Game");
        assert!(project.validate().is_ok());
    }

    #[test]
    fn test_ue5_config() {
        let toml_str = r#"
version = "1.0.0"

[project]
name = "UE5 Game"
id = "com.hummbl.ue5game"

[engine]
kind = "ue5"
uproject = "MyGame.uproject"
version = "5.4"

[build]
targets = ["linux-steam", "windows-steam"]
"#;
        let project: ForgeProject = toml::from_str(toml_str).unwrap();
        assert!(matches!(project.engine, EngineConfig::Ue5 { .. }));
        assert!(project.validate().is_ok());
    }

    #[test]
    fn test_invalid_version_rejected() {
        let toml_str = r#"
version = "not-semver"

[project]
name = "Bad Version"
id = "com.hummbl.bad"

[engine]
kind = "forge2d"
cargo_toml = "Cargo.toml"
"#;
        let project: ForgeProject = toml::from_str(toml_str).unwrap();
        assert!(project.validate().is_err());
    }

    #[test]
    fn test_empty_name_rejected() {
        let toml_str = r#"
version = "1.0.0"

[project]
name = ""
id = "com.hummbl.empty"

[engine]
kind = "forge2d"
cargo_toml = "Cargo.toml"
"#;
        let project: ForgeProject = toml::from_str(toml_str).unwrap();
        assert!(project.validate().is_err());
    }
}
