//! Build pipeline orchestration.
//!
//! Dispatches to the appropriate engine adapter based on the project's
//! engine configuration, then produces a BuildResult.

use forge_adapter::EngineAdapter;
use forge_core::config::EngineConfig;
use forge_core::ForgeProject;

/// Result of a successful build.
#[derive(Debug)]
pub struct BuildResult {
    pub platform: String,
    pub output_path: std::path::PathBuf,
    pub manifest: BuildManifest,
}

/// Manifest describing a build artifact.
#[derive(Debug, serde::Serialize)]
pub struct BuildManifest {
    pub project_name: String,
    pub version: String,
    pub platform: String,
    pub profile: String,
    pub build_hash: String,
    pub built_at: String,
}

/// Run the build pipeline for a project.
pub async fn build(
    project: &ForgeProject,
    platform: &str,
    profile: &str,
) -> anyhow::Result<BuildResult> {
    tracing::info!(
        "Building {} v{} for {} ({})",
        project.project.name,
        project.version,
        platform,
        profile
    );

    // Dispatch to the appropriate adapter based on engine kind
    let build_output = match &project.engine {
        EngineConfig::Forge2d { cargo_toml } => {
            tracing::info!("Dispatching to Forge2D adapter");
            // cargo_toml is relative to the forge.toml directory (current dir)
            // The adapter's project_dir is the parent of Cargo.toml
            let cargo_toml_path = std::path::Path::new(cargo_toml);
            let project_dir = cargo_toml_path
                .parent()
                .filter(|path| !path.as_os_str().is_empty())
                .unwrap_or(std::path::Path::new("."))
                .to_path_buf();
            let adapter = forge_adapter_forge2d::Forge2dAdapter::new(project_dir);
            adapter.build(project, platform, profile).await?
        }
        EngineConfig::Ue5 { .. } => {
            tracing::info!("Dispatching to UE5 adapter");
            #[cfg(feature = "ue5")]
            {
                let adapter = forge_adapter_ue5::Ue5Adapter::auto_detect()
                    .ok_or_else(|| anyhow::anyhow!("UE5 installation not found"))?;
                adapter.build(project, platform, profile).await?
            }
            #[cfg(not(feature = "ue5"))]
            {
                anyhow::bail!("UE5 adapter not compiled into this build (enable 'ue5' feature)")
            }
        }
    };

    tracing::info!(
        "Build complete: {} (hash: {})",
        build_output.output_path.display(),
        build_output.metadata.build_hash
    );

    Ok(BuildResult {
        platform: platform.to_string(),
        output_path: build_output.output_path,
        manifest: BuildManifest {
            project_name: project.project.name.clone(),
            version: project.version.clone(),
            platform: build_output.metadata.platform,
            profile: build_output.metadata.profile,
            build_hash: build_output.metadata.build_hash,
            built_at: build_output.metadata.built_at,
        },
    })
}
