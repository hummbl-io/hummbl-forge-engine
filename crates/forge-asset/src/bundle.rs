//! Asset bundling — package assets for distribution.
//!
//! Creates a compressed archive of all assets with a manifest for
//! integrity verification.

use std::path::Path;
use std::process::Command;

use crate::manifest::AssetManifest;

/// Result of bundling assets.
#[derive(Debug)]
pub struct BundleResult {
    pub bundle_path: std::path::PathBuf,
    pub manifest_path: std::path::PathBuf,
    pub asset_count: usize,
    pub total_size: u64,
    pub compressed_size: u64,
}

/// Bundle all assets in a directory into a compressed archive.
pub fn bundle_assets(
    assets_dir: &Path,
    output_dir: &Path,
    project_id: &str,
    version: &str,
) -> anyhow::Result<BundleResult> {
    if !assets_dir.exists() {
        anyhow::bail!("assets directory not found: {}", assets_dir.display());
    }

    std::fs::create_dir_all(output_dir)?;

    // Generate manifest
    let manifest = AssetManifest::from_directory(assets_dir, project_id, version)?;
    let manifest_path = output_dir.join("asset-manifest.json");
    manifest.write_to_file(&manifest_path)?;

    let asset_count = manifest.entries.len();
    let total_size = manifest.total_size();

    // Create tar.gz bundle
    let bundle_name = format!("{}-{}-assets.tar.gz", project_id.replace('.', "-"), version);
    let bundle_path = output_dir.join(&bundle_name);

    let status = Command::new("tar")
        .arg("czf")
        .arg(&bundle_path)
        .arg("-C")
        .arg(assets_dir.parent().unwrap_or(Path::new(".")))
        .arg(
            assets_dir
                .file_name()
                .unwrap_or(std::ffi::OsStr::new("assets")),
        )
        .status()
        .map_err(|e| anyhow::anyhow!("failed to run tar: {e}"))?;

    if !status.success() {
        anyhow::bail!("tar failed");
    }

    let compressed_size = std::fs::metadata(&bundle_path)?.len();

    tracing::info!(
        "Asset bundle: {} ({} assets, {} → {} bytes, {:.1}% compression)",
        bundle_path.display(),
        asset_count,
        total_size,
        compressed_size,
        if total_size > 0 {
            (1.0 - (compressed_size as f64 / total_size as f64)) * 100.0
        } else {
            0.0
        }
    );

    Ok(BundleResult {
        bundle_path,
        manifest_path,
        asset_count,
        total_size,
        compressed_size,
    })
}
