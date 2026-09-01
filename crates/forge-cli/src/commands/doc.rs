//! `forge doc` — generate project documentation.
//!
//! Generates API documentation for the project.

pub async fn run(open: bool) -> anyhow::Result<()> {
    let project = forge_core::ForgeProject::from_current_dir()
        .map_err(|e| anyhow::anyhow!("failed to load forge.toml: {e}"))?;

    tracing::info!(
        "Generating documentation for {} v{}",
        project.project.name,
        project.version
    );

    match &project.engine {
        forge_core::config::EngineConfig::Forge2d { cargo_toml, .. } => {
            run_cargo_doc(cargo_toml, open).await
        }
        forge_core::config::EngineConfig::Ue5 { .. } => {
            tracing::warn!("UE5 doc generation not yet implemented");
            Ok(())
        }
    }
}

/// Run cargo doc for a Forge2D project.
async fn run_cargo_doc(cargo_toml: &str, open: bool) -> anyhow::Result<()> {
    let cargo_toml_path = std::path::Path::new(cargo_toml);
    let project_dir = cargo_toml_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("invalid Cargo.toml path"))?;

    tracing::info!("Running cargo doc in {}", project_dir.display());

    let mut cmd = std::process::Command::new("cargo");
    cmd.arg("doc").arg("--no-deps").current_dir(project_dir);

    if open {
        cmd.arg("--open");
    }

    let output = cmd
        .output()
        .map_err(|e| anyhow::anyhow!("failed to run cargo doc: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("cargo doc failed: {stderr}");
    }

    tracing::info!("Documentation generated");
    Ok(())
}
