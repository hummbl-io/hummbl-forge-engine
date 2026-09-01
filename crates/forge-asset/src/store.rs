//! Content-addressable asset store.
//!
//! Assets are stored by SHA-256 hash. This enables deduplication
//! across games and asset lineage tracking.

use sha2::{Digest, Sha256};

/// Compute the SHA-256 hash of a file.
pub fn hash_file(path: &std::path::Path) -> std::io::Result<String> {
    let data = std::fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&data);
    Ok(hex::encode(hasher.finalize()))
}

/// An asset entry in the store.
#[derive(Debug, Clone, serde::Serialize)]
pub struct AssetEntry {
    pub hash: String,
    pub path: String,
    pub size: u64,
    pub format: String,
}
