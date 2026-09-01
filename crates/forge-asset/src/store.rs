//! Content-addressable asset store.
//!
//! Assets are stored by SHA-256 hash. This enables deduplication
//! across games and asset lineage tracking.

use std::path::Path;

use sha2::{Digest, Sha256};

/// Compute the SHA-256 hash of a file.
pub fn hash_file(path: &Path) -> std::io::Result<String> {
    let data = std::fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&data);
    Ok(hex::encode(hasher.finalize()))
}

/// An asset entry in the store.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AssetEntry {
    pub hash: String,
    pub path: String,
    pub size: u64,
    pub format: String,
}

/// Content-addressable asset store.
///
/// Assets are stored in a directory structure based on their hash:
///   store/ab/cd/abcdef1234567890...
///
/// This enables automatic deduplication — identical files share the same
/// storage location regardless of their original path.
pub struct AssetStore {
    root: std::path::PathBuf,
}

impl AssetStore {
    /// Create a new asset store at the given root directory.
    pub fn new(root: impl Into<std::path::PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Add a file to the store. Returns the hash and stored path.
    ///
    /// Symlinks are rejected to prevent information leakage from outside
    /// the store (BLUE TEAM hardening — addresses RED-TEAM-FINDING-4).
    pub fn add(&self, source: &Path) -> anyhow::Result<AssetEntry> {
        // BLUE TEAM: Reject symlinks to prevent info leak attacks
        let metadata = std::fs::symlink_metadata(source)
            .map_err(|e| anyhow::anyhow!("failed to stat file: {e}"))?;

        if metadata.file_type().is_symlink() {
            anyhow::bail!(
                "symlinks are not allowed in the asset store (path: {})",
                source.display()
            );
        }

        let hash = hash_file(source).map_err(|e| anyhow::anyhow!("failed to hash file: {e}"))?;
        let size = metadata.len();

        // Determine format from extension
        let format = source
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Store at <root>/<first 2 bytes>/<next 2 bytes>/<full hash>
        let (dir_a, dir_b) = (&hash[..2], &hash[2..4]);
        let store_dir = self.root.join(dir_a).join(dir_b);
        std::fs::create_dir_all(&store_dir)?;

        let stored_path = store_dir.join(&hash);
        if !stored_path.exists() {
            std::fs::copy(source, &stored_path)?;
            tracing::info!(
                "Added asset: {} → {}",
                source.display(),
                stored_path.display()
            );
        } else {
            tracing::debug!("Asset already in store: {hash}");
        }

        Ok(AssetEntry {
            hash,
            path: source.to_string_lossy().to_string(),
            size,
            format,
        })
    }

    /// Retrieve an asset by hash. Returns the path in the store.
    pub fn get(&self, hash: &str) -> Option<std::path::PathBuf> {
        let (dir_a, dir_b) = (&hash[..2], &hash[2..4]);
        let path = self.root.join(dir_a).join(dir_b).join(hash);
        if path.exists() {
            Some(path)
        } else {
            None
        }
    }

    /// Check if an asset exists in the store.
    pub fn contains(&self, hash: &str) -> bool {
        self.get(hash).is_some()
    }

    /// List all assets in the store.
    pub fn list(&self) -> anyhow::Result<Vec<AssetEntry>> {
        let mut entries = Vec::new();
        if !self.root.exists() {
            return Ok(entries);
        }

        collect_assets(&self.root, &self.root, &mut entries)?;
        entries.sort_by(|a, b| a.hash.cmp(&b.hash));
        Ok(entries)
    }

    /// Remove an asset by hash.
    pub fn remove(&self, hash: &str) -> anyhow::Result<bool> {
        if let Some(path) = self.get(hash) {
            std::fs::remove_file(&path)?;
            // Clean up empty directories
            if let Some(parent) = path.parent() {
                if std::fs::read_dir(parent)?.next().is_none() {
                    std::fs::remove_dir(parent)?;
                    if let Some(grandparent) = parent.parent() {
                        if std::fs::read_dir(grandparent)?.next().is_none() {
                            std::fs::remove_dir(grandparent)?;
                        }
                    }
                }
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Get the total size of all assets in the store.
    pub fn total_size(&self) -> anyhow::Result<u64> {
        let entries = self.list()?;
        Ok(entries.iter().map(|e| e.size).sum())
    }
}

/// Recursively collect assets from the store.
fn collect_assets(
    _base: &Path,
    current: &Path,
    entries: &mut Vec<AssetEntry>,
) -> anyhow::Result<()> {
    if !current.is_dir() {
        return Ok(());
    }
    for entry in std::fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_assets(_base, &path, entries)?;
        } else if path.is_file() {
            if let Some(hash) = path.file_name().and_then(|n| n.to_str()) {
                if hash.len() == 64 {
                    // SHA-256 hex string
                    let size = entry.metadata()?.len();
                    entries.push(AssetEntry {
                        hash: hash.to_string(),
                        path: path.to_string_lossy().to_string(),
                        size,
                        format: "stored".to_string(),
                    });
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_file() {
        let tmp = std::env::temp_dir().join("forge_asset_test.txt");
        std::fs::write(&tmp, "hello world").unwrap();
        let hash = hash_file(&tmp).unwrap();
        assert_eq!(hash.len(), 64); // SHA-256 hex length
        std::fs::remove_file(&tmp).ok();
    }

    #[test]
    fn test_asset_store_add_get() {
        let store_dir = std::env::temp_dir().join("forge_asset_store_test");
        std::fs::remove_dir_all(&store_dir).ok();
        std::fs::create_dir_all(&store_dir).ok();

        let store = AssetStore::new(&store_dir);

        // Create a test file
        let test_file = std::env::temp_dir().join("forge_test_asset.txt");
        std::fs::write(&test_file, "test asset content").unwrap();

        let entry = store.add(&test_file).unwrap();
        assert!(store.contains(&entry.hash));

        let retrieved = store.get(&entry.hash).unwrap();
        assert!(retrieved.exists());

        let listed = store.list().unwrap();
        assert_eq!(listed.len(), 1);

        let removed = store.remove(&entry.hash).unwrap();
        assert!(removed);
        assert!(!store.contains(&entry.hash));

        std::fs::remove_file(&test_file).ok();
        std::fs::remove_dir_all(&store_dir).ok();
    }
}
