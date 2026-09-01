//! Build pipeline orchestration.

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

    // TODO: dispatch to engine adapter
    anyhow::bail!("build pipeline not yet implemented — adapter dispatch pending")
}
