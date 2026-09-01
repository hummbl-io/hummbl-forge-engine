//! Blender headless export pipeline.
//!
//! Orchestrates Blender's command-line export to produce game-ready
//! assets from .blend source files.

/// Export a .blend file to glTF or FBX using Blender's headless mode.
pub async fn export(
    blend_path: &std::path::Path,
    output_path: &std::path::Path,
    format: ExportFormat,
    blender_path: Option<&str>,
) -> anyhow::Result<()> {
    let _blender = blender_path
        .map(|s| s.to_string())
        .unwrap_or_else(|| detect_blender().unwrap_or_else(|| "blender".to_string()));

    let format_flag = match format {
        ExportFormat::Gltf => "glTF",
        ExportFormat::Fbx => "FBX",
    };

    tracing::info!(
        "Exporting {} → {} as {}",
        blend_path.display(),
        output_path.display(),
        format_flag
    );

    // TODO: invoke Blender headless with a Python export script
    // blender --background blend_path --python-expr "..."
    anyhow::bail!("Blender export not yet implemented — subprocess invocation pending")
}

/// Asset export format.
#[derive(Debug, Clone, Copy)]
pub enum ExportFormat {
    Gltf,
    Fbx,
}

/// Try to detect Blender's installation path.
fn detect_blender() -> Option<String> {
    // Check common paths
    let candidates = [
        "blender",
        "/usr/bin/blender",
        "/snap/bin/blender",
        "C:\\Program Files\\Blender Foundation\\blender.exe",
    ];

    for candidate in &candidates {
        if std::process::Command::new(candidate)
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .is_ok()
        {
            return Some(candidate.to_string());
        }
    }

    None
}
