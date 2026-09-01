//! `forge build` — build a game for one or more platforms.

use forge_core::ForgeProject;

pub async fn run(
    engine_override: Option<&str>,
    platform_override: Option<&str>,
    profile: &str,
) -> anyhow::Result<()> {
    let project = ForgeProject::from_current_dir()
        .map_err(|e| anyhow::anyhow!("failed to load forge.toml: {e}"))?;

    project
        .validate()
        .map_err(|e| anyhow::anyhow!("invalid project: {e}"))?;

    tracing::info!(
        "Building {} v{} (engine: {}, profile: {})",
        project.project.name,
        project.version,
        engine_override.unwrap_or("from forge.toml"),
        profile
    );

    let targets = if let Some(p) = platform_override {
        vec![p.to_string()]
    } else {
        project.build.targets.clone()
    };

    if targets.is_empty() {
        anyhow::bail!(
            "no build targets specified (set [build].targets in forge.toml or use --platform)"
        );
    }

    for target in &targets {
        tracing::info!("  Target: {target}");
        // TODO: dispatch to the appropriate adapter via forge-build
    }

    tracing::info!("Build pipeline not yet implemented — this is a scaffold.");
    Ok(())
}
