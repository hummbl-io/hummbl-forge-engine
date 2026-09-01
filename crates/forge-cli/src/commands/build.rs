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

    let mut results = Vec::new();
    for target in &targets {
        match forge_build::pipeline::build(&project, target, profile).await {
            Ok(result) => {
                tracing::info!(
                    "  ✓ {} → {} (hash: {})",
                    target,
                    result.output_path.display(),
                    result.manifest.build_hash
                );
                results.push(result);
            }
            Err(e) => {
                tracing::error!("  ✗ {} → {e}", target);
                return Err(e);
            }
        }
    }

    tracing::info!(
        "Build complete: {}/{} targets succeeded",
        results.len(),
        targets.len()
    );

    // Write build manifest
    if let Some(first) = results.first() {
        let manifest_path = std::path::Path::new("build-manifest.json");
        let manifest_json = serde_json::to_string_pretty(&first.manifest)?;
        std::fs::write(manifest_path, manifest_json)?;
        tracing::info!("Manifest written to {}", manifest_path.display());
    }

    Ok(())
}
