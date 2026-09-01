//! forge-asset: content-addressable asset store + Blender pipeline.
//!
//! Provides:
//! - Content-addressable asset store (SHA-256 based deduplication)
//! - Blender headless export (glTF, FBX)
//! - Asset bundling for distribution
//! - Asset manifest generation

pub mod blender;
pub mod bundle;
pub mod manifest;
pub mod store;

pub use blender::{detect_blender, export, ExportFormat};
pub use bundle::{bundle_assets, BundleResult};
pub use manifest::{AssetManifest, AssetManifestEntry};
pub use store::{hash_file, AssetEntry, AssetStore};
