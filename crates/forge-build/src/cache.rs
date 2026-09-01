//! Local build cache — caches build artifacts by source hash.
//!
//! Enables incremental builds by caching compiled outputs keyed by
//! the hash of the source files and build configuration.

use std::path::Path;
use std::time::SystemTime;

use sha2::{Digest, Sha256};

/// A build cache entry.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CacheEntry {
    pub cache_key: String,
    pub source_hash: String,
    pub platform: String,
    pub profile: String,
    pub artifact_path: String,
    pub artifact_hash: String,
    pub cached_at: String,
    pub size: u64,
}

/// Local build cache.
pub struct BuildCache {
    cache_dir: std::path::PathBuf,
}

impl BuildCache {
    /// Create a new build cache at the given directory.
    pub fn new(cache_dir: impl Into<std::path::PathBuf>) -> Self {
        Self {
            cache_dir: cache_dir.into(),
        }
    }

    /// Compute a cache key for a build configuration.
    pub fn compute_key(
        &self,
        project_id: &str,
        version: &str,
        platform: &str,
        profile: &str,
        source_hash: &str,
    ) -> String {
        let mut hasher = Sha256::new();
        hasher.update(project_id.as_bytes());
        hasher.update(version.as_bytes());
        hasher.update(platform.as_bytes());
        hasher.update(profile.as_bytes());
        hasher.update(source_hash.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Look up a cached build.
    pub fn get(&self, cache_key: &str) -> Option<CacheEntry> {
        let entry_path = self.cache_dir.join(format!("{cache_key}.json"));
        if !entry_path.exists() {
            return None;
        }
        let contents = std::fs::read_to_string(&entry_path).ok()?;
        let entry: CacheEntry = serde_json::from_str(&contents).ok()?;

        // Verify the artifact still exists
        if !Path::new(&entry.artifact_path).exists() {
            return None;
        }

        tracing::debug!("Cache hit: {cache_key}");
        Some(entry)
    }

    /// Store a build in the cache.
    pub fn put(&self, entry: CacheEntry) -> anyhow::Result<()> {
        std::fs::create_dir_all(&self.cache_dir)?;
        let entry_path = self.cache_dir.join(format!("{}.json", entry.cache_key));
        let json = serde_json::to_string_pretty(&entry)?;
        std::fs::write(&entry_path, json)?;
        tracing::debug!("Cached build: {}", entry.cache_key);
        Ok(())
    }

    /// Remove a cached build.
    pub fn remove(&self, cache_key: &str) -> anyhow::Result<bool> {
        let entry_path = self.cache_dir.join(format!("{cache_key}.json"));
        if entry_path.exists() {
            std::fs::remove_file(&entry_path)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Clear the entire cache.
    pub fn clear(&self) -> anyhow::Result<()> {
        if self.cache_dir.exists() {
            std::fs::remove_dir_all(&self.cache_dir)?;
            std::fs::create_dir_all(&self.cache_dir)?;
        }
        Ok(())
    }

    /// Get the total size of the cache.
    pub fn total_size(&self) -> anyhow::Result<u64> {
        if !self.cache_dir.exists() {
            return Ok(0);
        }
        let mut total = 0u64;
        for entry in std::fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            total += entry.metadata()?.len();
        }
        Ok(total)
    }

    /// List all cache entries.
    pub fn list(&self) -> anyhow::Result<Vec<CacheEntry>> {
        if !self.cache_dir.exists() {
            return Ok(Vec::new());
        }
        let mut entries = Vec::new();
        for entry in std::fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            if entry.path().extension().and_then(|e| e.to_str()) == Some("json") {
                let contents = std::fs::read_to_string(entry.path())?;
                if let Ok(cache_entry) = serde_json::from_str::<CacheEntry>(&contents) {
                    entries.push(cache_entry);
                }
            }
        }
        entries.sort_by(|a, b| a.cached_at.cmp(&b.cached_at));
        Ok(entries)
    }

    /// Prune entries older than the given age.
    pub fn prune_older_than(&self, max_age_secs: u64) -> anyhow::Result<usize> {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut pruned = 0;
        for entry in self.list()? {
            if let Ok(cached_at) = entry.cached_at.parse::<u64>() {
                if now - cached_at > max_age_secs {
                    self.remove(&entry.cache_key)?;
                    pruned += 1;
                }
            }
        }
        Ok(pruned)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_deterministic() {
        let cache = BuildCache::new("/tmp/forge-cache-test");
        let key1 = cache.compute_key("test", "0.1.0", "linux", "release", "abc123");
        let key2 = cache.compute_key("test", "0.1.0", "linux", "release", "abc123");
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_cache_key_differs_by_platform() {
        let cache = BuildCache::new("/tmp/forge-cache-test");
        let key1 = cache.compute_key("test", "0.1.0", "linux", "release", "abc123");
        let key2 = cache.compute_key("test", "0.1.0", "windows", "release", "abc123");
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_cache_put_get() {
        let cache_dir = std::env::temp_dir().join("forge_build_cache_test");
        std::fs::remove_dir_all(&cache_dir).ok();

        let cache = BuildCache::new(&cache_dir);
        let entry = CacheEntry {
            cache_key: "testkey123".to_string(),
            source_hash: "abc".to_string(),
            platform: "linux".to_string(),
            profile: "release".to_string(),
            artifact_path: "/tmp/test-artifact".to_string(),
            artifact_hash: "def".to_string(),
            cached_at: "1234567890".to_string(),
            size: 1024,
        };

        cache.put(entry.clone()).unwrap();
        let retrieved = cache.get("testkey123");
        // The artifact doesn't exist, so get() returns None
        assert!(retrieved.is_none());

        // Create the artifact file
        std::fs::write("/tmp/test-artifact", "test").unwrap();
        let retrieved = cache.get("testkey123").unwrap();
        assert_eq!(retrieved.platform, "linux");
        assert_eq!(retrieved.size, 1024);

        cache.clear().unwrap();
        std::fs::remove_file("/tmp/test-artifact").ok();
        std::fs::remove_dir_all(&cache_dir).ok();
    }
}
