//! Blender headless export pipeline.
//!
//! Orchestrates Blender's command-line export to produce game-ready
//! assets from .blend source files.

use std::path::Path;
use std::process::Command;

/// Asset export format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Gltf,
    GltfBinary,
    Fbx,
    Obj,
}

impl ExportFormat {
    fn blender_export_format(&self) -> &'static str {
        match self {
            Self::Gltf => "GLTF_SEPARATE",
            Self::GltfBinary => "GLB",
            Self::Fbx => "FBX",
            Self::Obj => "OBJ",
        }
    }

    fn file_extension(&self) -> &'static str {
        match self {
            Self::Gltf => "gltf",
            Self::GltfBinary => "glb",
            Self::Fbx => "fbx",
            Self::Obj => "obj",
        }
    }
}

/// Export a .blend file to glTF or FBX using Blender's headless mode.
pub async fn export(
    blend_path: &Path,
    output_path: &Path,
    format: ExportFormat,
    blender_path: Option<&str>,
) -> anyhow::Result<()> {
    let blender = blender_path
        .map(|s| s.to_string())
        .unwrap_or_else(|| detect_blender().unwrap_or_else(|| "blender".to_string()));

    if !blend_path.exists() {
        anyhow::bail!("Blend file not found: {}", blend_path.display());
    }

    // Ensure output directory exists
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let format_flag = format.blender_export_format();
    let output_str = output_path.to_string_lossy();

    // Build the Python export script
    let python_script = format!(
        r#"
import bpy
bpy.ops.wm.open_mainfile(filepath='{blend}')
bpy.ops.export_scene.gltf(filepath='{output}', export_format='{format}')
print("Export complete: {output}")
"#,
        blend = blend_path.display().to_string().replace('\\', "/"),
        output = output_str.replace('\\', "/"),
        format = format_flag,
    );

    tracing::info!(
        "Exporting {} → {} as {}",
        blend_path.display(),
        output_path.display(),
        format.file_extension()
    );

    let output = Command::new(&blender)
        .arg("--background")
        .arg(blend_path)
        .arg("--python-expr")
        .arg(&python_script)
        .output()
        .map_err(|e| anyhow::anyhow!("failed to run Blender: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        anyhow::bail!("Blender export failed:\nstdout: {stdout}\nstderr: {stderr}");
    }

    // Verify the output file was created
    if !output_path.exists() {
        // For glTF separate, the output might be a .gltf + .bin + textures
        let alt_path = output_path.with_extension(format.file_extension());
        if !alt_path.exists() {
            anyhow::bail!(
                "export succeeded but output file not found at {}",
                output_path.display()
            );
        }
    }

    tracing::info!("Export complete: {}", output_path.display());
    Ok(())
}

/// Try to detect Blender's installation path.
pub fn detect_blender() -> Option<String> {
    let candidates = [
        "blender",
        "/usr/bin/blender",
        "/snap/bin/blender",
        "/usr/local/bin/blender",
        "C:\\Program Files\\Blender Foundation\\Blender 4.2\\blender.exe",
        "C:\\Program Files\\Blender Foundation\\Blender 4.1\\blender.exe",
        "C:\\Program Files\\Blender Foundation\\Blender 3.6\\blender.exe",
    ];

    for candidate in &candidates {
        if Command::new(candidate)
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

/// Batch export all .blend files in a directory.
pub async fn export_directory(
    source_dir: &Path,
    output_dir: &Path,
    format: ExportFormat,
    blender_path: Option<&str>,
) -> anyhow::Result<Vec<std::path::PathBuf>> {
    if !source_dir.is_dir() {
        anyhow::bail!("source directory not found: {}", source_dir.display());
    }

    std::fs::create_dir_all(output_dir)?;
    let mut exported = Vec::new();

    for entry in std::fs::read_dir(source_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("blend") {
            let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("asset");
            let output_path = output_dir.join(format!("{}.{}", stem, format.file_extension()));
            export(&path, &output_path, format, blender_path).await?;
            exported.push(output_path);
        }
    }

    tracing::info!("Batch export: {}/{} files", exported.len(), exported.len());
    Ok(exported)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_format_extensions() {
        assert_eq!(ExportFormat::Gltf.file_extension(), "gltf");
        assert_eq!(ExportFormat::GltfBinary.file_extension(), "glb");
        assert_eq!(ExportFormat::Fbx.file_extension(), "fbx");
        assert_eq!(ExportFormat::Obj.file_extension(), "obj");
    }

    #[test]
    fn test_export_format_blender_flags() {
        assert_eq!(ExportFormat::Gltf.blender_export_format(), "GLTF_SEPARATE");
        assert_eq!(ExportFormat::GltfBinary.blender_export_format(), "GLB");
        assert_eq!(ExportFormat::Fbx.blender_export_format(), "FBX");
    }
}
