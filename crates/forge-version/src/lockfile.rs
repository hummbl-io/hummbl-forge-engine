//! Lockfile generation and parsing.
//!
//! `forge.lock` pins exact versions of engine, adapter, game, and mods —
//! like Cargo.lock but for a Forge Engine project.

use std::path::Path;

use serde::{Deserialize, Serialize};

use forge_core::ForgeProject;

/// Current lockfile format version.
pub const FORMAT_VERSION: u32 = 1;

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
    /// When the lockfile was generated (ISO 8601)
    pub generated_at: String,
    /// Integrity hash — SHA-256 of all fields above (BLUE TEAM hardening).
    /// Detects manual tampering with any lockfile field.
    #[serde(default)]
    pub integrity_hash: Option<String>,
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

impl Lockfile {
    /// Generate a lockfile from a Forge project.
    pub fn from_project(project: &ForgeProject) -> anyhow::Result<Self> {
        let engine = match &project.engine {
            forge_core::config::EngineConfig::Forge2d { .. } => EnginePin {
                kind: "forge2d".to_string(),
                version: "0.1.0".to_string(),
                adapter_version: "0.1.0".to_string(),
            },
            forge_core::config::EngineConfig::Ue5 { version, .. } => EnginePin {
                kind: "ue5".to_string(),
                version: version.clone(),
                adapter_version: "0.1.0".to_string(),
            },
        };

        // Hash assets
        let assets = hash_assets(&project.assets.assets_dir)?;

        let mut lockfile = Self {
            format_version: FORMAT_VERSION,
            project_id: project.project.id.clone(),
            project_version: project.version.clone(),
            engine,
            mods: Vec::new(), // populated when mods are installed
            assets,
            generated_at: current_timestamp(),
            integrity_hash: None, // computed below
        };

        // BLUE TEAM: Compute integrity hash over all fields
        lockfile.integrity_hash = Some(lockfile.compute_integrity_hash());

        Ok(lockfile)
    }

    /// Compute the integrity hash over all lockfile fields (except the hash itself).
    pub fn compute_integrity_hash(&self) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(self.format_version.to_string().as_bytes());
        hasher.update(self.project_id.as_bytes());
        hasher.update(self.project_version.as_bytes());
        hasher.update(self.engine.kind.as_bytes());
        hasher.update(self.engine.version.as_bytes());
        hasher.update(self.engine.adapter_version.as_bytes());
        for m in &self.mods {
            hasher.update(m.id.as_bytes());
            hasher.update(m.version.as_bytes());
            hasher.update(m.hash.as_bytes());
        }
        for a in &self.assets {
            hasher.update(a.path.as_bytes());
            hasher.update(a.hash.as_bytes());
        }
        hasher.update(self.generated_at.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Verify the integrity hash matches the current content.
    /// Returns false if the lockfile has been tampered with.
    pub fn verify_integrity(&self) -> bool {
        match &self.integrity_hash {
            Some(expected) => {
                let computed = self.compute_integrity_hash();
                computed == *expected
            }
            None => false, // missing integrity hash = untrusted
        }
    }

    /// Write the lockfile to a file (TOML format).
    pub fn write_to_file(&self, path: &Path) -> anyhow::Result<()> {
        let toml = toml::to_string_pretty(self)
            .map_err(|e| anyhow::anyhow!("failed to serialize lockfile: {e}"))?;
        std::fs::write(path, toml)?;
        Ok(())
    }

    /// Read a lockfile from a file.
    pub fn read_from_file(path: &Path) -> anyhow::Result<Self> {
        let contents = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("failed to read lockfile: {e}"))?;
        let lockfile: Self = toml::from_str(&contents)
            .map_err(|e| anyhow::anyhow!("failed to parse lockfile: {e}"))?;
        Ok(lockfile)
    }

    /// Check if the lockfile is up to date with the project.
    /// Also verifies the integrity hash (BLUE TEAM hardening).
    pub fn is_up_to_date(&self, project: &ForgeProject) -> bool {
        // BLUE TEAM: Verify integrity hash first
        if !self.verify_integrity() {
            tracing::warn!("lockfile integrity hash mismatch — file may have been tampered with");
            return false;
        }

        if self.project_id != project.project.id {
            return false;
        }
        if self.project_version != project.version {
            return false;
        }
        // Check engine kind matches
        let expected_kind = match &project.engine {
            forge_core::config::EngineConfig::Forge2d { .. } => "forge2d",
            forge_core::config::EngineConfig::Ue5 { .. } => "ue5",
        };
        if self.engine.kind != expected_kind {
            return false;
        }
        true
    }
}

/// Hash all files in the assets directory.
fn hash_assets(assets_dir: &str) -> anyhow::Result<Vec<AssetPin>> {
    use sha2::{Digest, Sha256};

    let dir = Path::new(assets_dir);
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut assets = Vec::new();
    let mut entries = Vec::new();
    collect_files(dir, dir, &mut entries)?;

    for (rel_path, full_path) in entries {
        let data = std::fs::read(&full_path)
            .map_err(|e| anyhow::anyhow!("failed to read asset {}: {e}", full_path.display()))?;
        let mut hasher = Sha256::new();
        hasher.update(&data);
        let hash = hex::encode(hasher.finalize());
        assets.push(AssetPin {
            path: rel_path,
            hash,
        });
    }

    // Sort for deterministic output
    assets.sort_by(|a, b| a.path.cmp(&b.path));

    Ok(assets)
}

/// Recursively collect files with their relative paths.
fn collect_files(
    base: &Path,
    current: &Path,
    entries: &mut Vec<(String, std::path::PathBuf)>,
) -> anyhow::Result<()> {
    if !current.is_dir() {
        return Ok(());
    }
    for entry in std::fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_files(base, &path, entries)?;
        } else if path.is_file() {
            let rel = path
                .strip_prefix(base)
                .map_err(|e| anyhow::anyhow!("path error: {e}"))?
                .to_string_lossy()
                .to_string();
            entries.push((rel, path));
        }
    }
    Ok(())
}

fn current_timestamp() -> String {
    // Simple ISO 8601-ish timestamp
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{now}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lockfile_serialization() {
        let lockfile = Lockfile {
            format_version: 1,
            project_id: "com.hummbl.test".to_string(),
            project_version: "0.1.0".to_string(),
            engine: EnginePin {
                kind: "forge2d".to_string(),
                version: "0.1.0".to_string(),
                adapter_version: "0.1.0".to_string(),
            },
            mods: vec![ModPin {
                id: "test-mod".to_string(),
                version: "1.0.0".to_string(),
                hash: "abc123".to_string(),
            }],
            assets: vec![AssetPin {
                path: "textures/sprite.png".to_string(),
                hash: "def456".to_string(),
            }],
            generated_at: "1234567890".to_string(),
            integrity_hash: None,
        };

        let toml = toml::to_string(&lockfile).unwrap();
        let parsed: Lockfile = toml::from_str(&toml).unwrap();
        assert_eq!(parsed.project_id, "com.hummbl.test");
        assert_eq!(parsed.engine.kind, "forge2d");
        assert_eq!(parsed.mods.len(), 1);
        assert_eq!(parsed.assets.len(), 1);
    }

    #[test]
    fn test_integrity_hash_tamper_detection() {
        let mut lockfile = Lockfile {
            format_version: 1,
            project_id: "com.hummbl.test".to_string(),
            project_version: "0.1.0".to_string(),
            engine: EnginePin {
                kind: "forge2d".to_string(),
                version: "0.1.0".to_string(),
                adapter_version: "0.1.0".to_string(),
            },
            mods: vec![],
            assets: vec![],
            generated_at: "1234567890".to_string(),
            integrity_hash: None,
        };

        // Compute and set the integrity hash
        lockfile.integrity_hash = Some(lockfile.compute_integrity_hash());

        // Verify it passes
        assert!(lockfile.verify_integrity());

        // Tamper with the version
        lockfile.project_version = "99.99.99".to_string();

        // Verify it now fails
        assert!(
            !lockfile.verify_integrity(),
            "tampered lockfile should fail integrity check"
        );
    }

    #[test]
    fn test_empty_assets_dir() {
        let assets = hash_assets("/nonexistent/path").unwrap();
        assert!(assets.is_empty());
    }
}
