//! Direct publishing — copies build artifacts to a local dist directory.
//!
//! The simplest publisher: no store, no upload, just organize the output.
//! Useful for testing the pipeline end-to-end and for self-hosted distribution.

use std::path::Path;

use crate::store::UploadReceipt;

/// Package a binary + assets into a local dist directory.
pub async fn package(
    binary_path: &Path,
    project_name: &str,
    version: &str,
    assets_dir: Option<&Path>,
    output_dir: &Path,
) -> anyhow::Result<std::path::PathBuf> {
    let dist = output_dir.join(format!("{project_name}-{version}"));
    std::fs::create_dir_all(&dist)?;

    // Copy binary
    let binary_name = binary_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow::anyhow!("invalid binary path"))?;
    let dest_binary = dist.join(binary_name);
    std::fs::copy(binary_path, &dest_binary)?;
    make_executable(&dest_binary)?;

    // Copy assets if present
    if let Some(assets) = assets_dir {
        if assets.exists() {
            let dest_assets = dist.join("assets");
            copy_dir_recursive(assets, &dest_assets)?;
        }
    }

    // Write a README
    let readme = format!(
        "# {project_name} v{version}\n\n\
         Built with HUMMBL Forge Engine.\n\n\
         ## Run\n\n\
         ```bash\n./{binary_name}\n```\n"
    );
    std::fs::write(dist.join("README.md"), readme)?;

    tracing::info!("Direct package created: {}", dist.display());
    Ok(dist)
}

pub fn local_receipt(_package_path: &Path) -> UploadReceipt {
    UploadReceipt {
        store: "direct".to_string(),
        uploaded_at: current_timestamp(),
        build_hash: String::new(),
        store_build_id: None,
        status: "packaged".to_string(),
    }
}

fn make_executable(path: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(path)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(path, perms)?;
    }
    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
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
