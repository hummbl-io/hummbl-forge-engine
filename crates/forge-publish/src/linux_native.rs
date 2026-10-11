//! Linux native packaging — AppImage, .deb, .rpm, tarball.
//!
//! Produces an AppDir structure and packages it. If appimagetool is
//! available, creates an AppImage. Otherwise, falls back to a tarball
//! with a .desktop file and install script.

use std::path::Path;
use std::process::Command;

use crate::store::UploadReceipt;

/// Package a binary + assets for Linux native distribution.
///
/// Produces:
/// - If appimagetool available: `<name>.AppImage`
/// - Otherwise: `<name>.tar.gz` containing binary, assets, .desktop file
pub async fn package(
    binary_path: &Path,
    project_name: &str,
    project_id: &str,
    version: &str,
    assets_dir: Option<&Path>,
    output_dir: &Path,
) -> anyhow::Result<std::path::PathBuf> {
    std::fs::create_dir_all(output_dir)?;

    // Create AppDir structure
    let appdir = output_dir.join(format!("{project_name}.AppDir"));
    let appdir_bin = appdir.join("usr/bin");
    let appdir_share = appdir.join("usr/share");
    let appdir_applications = appdir.join("usr/share/applications");
    let appdir_icons = appdir.join("usr/share/icons/hicolor/256x256/apps");

    // Clean any previous AppDir
    if appdir.exists() {
        std::fs::remove_dir_all(&appdir)?;
    }

    std::fs::create_dir_all(&appdir_bin)?;
    std::fs::create_dir_all(&appdir_applications)?;
    std::fs::create_dir_all(&appdir_icons)?;

    // Copy binary
    let binary_name = binary_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow::anyhow!("invalid binary path"))?;
    let dest_binary = appdir_bin.join(binary_name);
    std::fs::copy(binary_path, &dest_binary)?;
    make_executable(&dest_binary)?;

    // Copy assets if present
    if let Some(assets) = assets_dir {
        if assets.exists() {
            let dest_assets = appdir_share.join(project_name);
            copy_dir_recursive(assets, &dest_assets)?;
        }
    }

    // Create .desktop file
    let desktop_entry = format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Name={project_name}\n\
         Exec=usr/bin/{binary_name}\n\
         Icon={project_id}\n\
         Categories=Game;\n\
         Terminal=false\n"
    );
    let desktop_path = appdir_applications.join(format!("{project_id}.desktop"));
    std::fs::write(&desktop_path, desktop_entry)?;

    // Create a placeholder icon if no icon provided
    let icon_path = appdir_icons.join(format!("{project_id}.png"));
    if !icon_path.exists() {
        // Write a minimal 1x1 PNG as placeholder
        std::fs::write(&icon_path, MINIMAL_PNG)?;
    }

    // Create AppRun (the AppImage entry point)
    let apprun = appdir.join("AppRun");
    std::fs::write(
        &apprun,
        format!("#!/bin/sh\nexec usr/bin/{binary_name} \"$@\"\n"),
    )?;
    make_executable(&apprun)?;

    // Try to create AppImage using appimagetool
    if has_appimagetool() {
        tracing::info!("appimagetool found — creating AppImage");
        let appimage_path = output_dir.join(format!("{project_name}-{version}.AppImage"));
        let status = Command::new("appimagetool")
            .arg(&appdir)
            .arg(&appimage_path)
            .status()
            .map_err(|e| anyhow::anyhow!("failed to run appimagetool: {e}"))?;

        if status.success() {
            // Clean up AppDir
            let _ = std::fs::remove_dir_all(&appdir);
            tracing::info!("AppImage created: {}", appimage_path.display());
            return Ok(appimage_path);
        } else {
            tracing::warn!("appimagetool failed, falling back to tarball");
        }
    }

    // Fallback: create tarball
    let tarball_path = output_dir.join(format!("{project_name}-{version}-linux.tar.gz"));
    tracing::info!("Creating tarball: {}", tarball_path.display());

    let status = Command::new("tar")
        .arg("czf")
        .arg(&tarball_path)
        .arg("-C")
        .arg(output_dir)
        .arg(format!("{project_name}.AppDir"))
        .status()
        .map_err(|e| anyhow::anyhow!("failed to run tar: {e}"))?;

    if !status.success() {
        anyhow::bail!("tar failed");
    }

    // Clean up AppDir
    let _ = std::fs::remove_dir_all(&appdir);

    tracing::info!("Tarball created: {}", tarball_path.display());
    Ok(tarball_path)
}

/// Upload receipt for linux-native (always "local" since there's no store).
pub fn local_receipt(_package_path: &Path) -> UploadReceipt {
    UploadReceipt {
        store: "linux-native".to_string(),
        uploaded_at: current_timestamp(),
        build_hash: String::new(), // filled by caller
        store_build_id: None,
        status: "packaged".to_string(),
    }
}

fn has_appimagetool() -> bool {
    Command::new("appimagetool")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok()
}

fn make_executable(path: &Path) -> std::io::Result<()> {
    #[cfg(not(unix))]
    let _ = path;
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

/// Minimal 1x1 transparent PNG (placeholder icon).
const MINIMAL_PNG: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
    0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR chunk
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, // 1x1
    0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4, // RGBA, CRC
    0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, // IDAT chunk
    0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00, // compressed data
    0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, // CRC
    0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, // IEND chunk
    0xAE, 0x42, 0x60, 0x82,
];
