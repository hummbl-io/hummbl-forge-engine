//! Asset manifest — tracks all assets in a project with their hashes.
//!
//! The manifest is generated during the build process and embedded in
//! the final package. It enables:
//! - Asset verification (check integrity after install)
//! - Differential updates (only download changed assets)
//! - Asset deduplication across versions

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::store::hash_file;

/// A single entry in the asset manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetManifestEntry {
    pub path: String,
    pub hash: String,
    pub size: u64,
    pub format: String,
}

/// The full asset manifest for a project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetManifest {
    pub project_id: String,
    pub version: String,
    pub entries: Vec<AssetManifestEntry>,
    pub generated_at: String,
}

impl AssetManifest {
    /// Generate a manifest from an assets directory.
    pub fn from_directory(
        assets_dir: &Path,
        project_id: &str,
        version: &str,
    ) -> anyhow::Result<Self> {
        let mut entries = Vec::new();

        if assets_dir.exists() {
            collect_assets(assets_dir, assets_dir, &mut entries)?;
        }

        // Sort for deterministic output
        entries.sort_by(|a, b| a.path.cmp(&b.path));

        Ok(Self {
            project_id: project_id.to_string(),
            version: version.to_string(),
            entries,
            generated_at: current_timestamp(),
        })
    }

    /// Write the manifest to a JSON file.
    pub fn write_to_file(&self, path: &Path) -> anyhow::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Read a manifest from a JSON file.
    pub fn read_from_file(path: &Path) -> anyhow::Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        let manifest: Self = serde_json::from_str(&contents)?;
        Ok(manifest)
    }

    /// Verify that all assets in the manifest exist and match their hashes.
    pub fn verify(&self, assets_dir: &Path) -> anyhow::Result<Vec<VerificationResult>> {
        let mut results = Vec::new();
        for entry in &self.entries {
            let path = assets_dir.join(&entry.path);
            if !path.exists() {
                results.push(VerificationResult {
                    path: entry.path.clone(),
                    status: VerificationStatus::Missing,
                    expected_hash: entry.hash.clone(),
                    actual_hash: None,
                });
                continue;
            }
            let actual_hash = hash_file(&path)
                .map_err(|e| anyhow::anyhow!("failed to hash {}: {e}", entry.path))?;
            let status = if actual_hash == entry.hash {
                VerificationStatus::Ok
            } else {
                VerificationStatus::HashMismatch
            };
            results.push(VerificationResult {
                path: entry.path.clone(),
                status,
                expected_hash: entry.hash.clone(),
                actual_hash: Some(actual_hash),
            });
        }
        Ok(results)
    }

    /// Get the total size of all assets.
    pub fn total_size(&self) -> u64 {
        self.entries.iter().map(|e| e.size).sum()
    }
}

/// Result of verifying a single asset.
#[derive(Debug, Clone)]
pub struct VerificationResult {
    pub path: String,
    pub status: VerificationStatus,
    pub expected_hash: String,
    pub actual_hash: Option<String>,
}

/// Status of an asset verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationStatus {
    Ok,
    Missing,
    HashMismatch,
}

/// Recursively collect assets with their hashes.
fn collect_assets(
    base: &Path,
    current: &Path,
    entries: &mut Vec<AssetManifestEntry>,
) -> anyhow::Result<()> {
    if !current.is_dir() {
        return Ok(());
    }
    for entry in std::fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_assets(base, &path, entries)?;
        } else if path.is_file() {
            let rel = path
                .strip_prefix(base)
                .map_err(|e| anyhow::anyhow!("path error: {e}"))?
                .to_string_lossy()
                .to_string();
            let hash =
                hash_file(&path).map_err(|e| anyhow::anyhow!("failed to hash {rel}: {e}"))?;
            let size = entry.metadata()?.len();
            let format = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("unknown")
                .to_string();
            entries.push(AssetManifestEntry {
                path: rel,
                hash,
                size,
                format,
            });
        }
    }
    Ok(())
}

fn current_timestamp() -> String {
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
    fn test_manifest_from_empty_dir() {
        let manifest =
            AssetManifest::from_directory(std::path::Path::new("/nonexistent"), "test", "0.1.0")
                .unwrap();
        assert_eq!(manifest.entries.len(), 0);
    }

    #[test]
    fn test_manifest_serialization() {
        let manifest = AssetManifest {
            project_id: "test".to_string(),
            version: "0.1.0".to_string(),
            entries: vec![AssetManifestEntry {
                path: "textures/sprite.png".to_string(),
                hash: "abc123".to_string(),
                size: 1024,
                format: "png".to_string(),
            }],
            generated_at: "1234567890".to_string(),
        };
        let json = serde_json::to_string(&manifest).unwrap();
        let parsed: AssetManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.entries.len(), 1);
        assert_eq!(parsed.total_size(), 1024);
    }
}
