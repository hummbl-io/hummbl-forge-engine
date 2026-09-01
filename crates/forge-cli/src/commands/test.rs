//! `forge test` — run project tests.
//!
//! Runs the game's test suite (cargo test for Forge2D projects,
//! or UE5 automation tests for UE5 projects).

pub async fn run() -> anyhow::Result<()> {
    let project = forge_core::ForgeProject::from_current_dir()
        .map_err(|e| anyhow::anyhow!("failed to load forge.toml: {e}"))?;

    tracing::info!(
        "Running tests for {} v{}",
        project.project.name,
        project.version
    );

    match &project.engine {
        forge_core::config::EngineConfig::Forge2d { cargo_toml, .. } => {
            run_cargo_tests(cargo_toml).await
        }
        forge_core::config::EngineConfig::Ue5 { .. } => {
            tracing::warn!("UE5 test runner not yet implemented — use the UE5 editor");
            Ok(())
        }
    }
}

/// Run cargo tests for a Forge2D project.
async fn run_cargo_tests(cargo_toml: &str) -> anyhow::Result<()> {
    let cargo_toml_path = std::path::Path::new(cargo_toml);
    let project_dir = cargo_toml_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("invalid Cargo.toml path"))?;

    tracing::info!("Running cargo test in {}", project_dir.display());

    let output = std::process::Command::new("cargo")
        .arg("test")
        .current_dir(project_dir)
        .output()
        .map_err(|e| anyhow::anyhow!("failed to run cargo test: {e}"))?;

    // Print test output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    print!("{stdout}");
    eprint!("{stderr}");

    if !output.status.success() {
        anyhow::bail!("tests failed with status: {output:?}");
    }

    tracing::info!("All tests passed");
    Ok(())
}
