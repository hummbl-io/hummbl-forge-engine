//! `forge publish` — publish a build to a store.

use forge_core::ForgeProject;

pub async fn run(target: &str, dry_run: bool) -> anyhow::Result<()> {
    let project = ForgeProject::from_current_dir()
        .map_err(|e| anyhow::anyhow!("failed to load forge.toml: {e}"))?;

    project
        .validate()
        .map_err(|e| anyhow::anyhow!("invalid project: {e}"))?;

    tracing::info!(
        "Publishing {} v{} to {target} (dry_run: {dry_run})",
        project.project.name,
        project.version
    );

    // Find the build artifact — look for build-manifest.json
    let manifest_path = std::path::Path::new("build-manifest.json");
    if !manifest_path.exists() {
        anyhow::bail!("no build manifest found — run `forge build` first");
    }

    let manifest_json = std::fs::read_to_string(manifest_path)?;
    let manifest: serde_json::Value = serde_json::from_str(&manifest_json)?;

    // The build output path is the binary from the last build
    // We need to find it based on the platform
    let platform = manifest
        .get("platform")
        .and_then(|v| v.as_str())
        .unwrap_or("linux-native");

    // Find the binary in the target directory
    // For Forge2D: target/release/<binary_name>
    // For UE5: would be in the archived directory
    let binary_path = find_build_output(&project, platform)?;

    if !binary_path.exists() {
        anyhow::bail!(
            "build output not found at {} — run `forge build` first",
            binary_path.display()
        );
    }

    let output_dir = std::path::Path::new("dist");
    let assets_dir = std::path::Path::new(&project.assets.assets_dir);
    let assets = if assets_dir.exists() {
        Some(assets_dir)
    } else {
        None
    };

    let package_path = match target {
        "linux-native" => {
            forge_publish::linux_native::package(
                &binary_path,
                &project.project.name,
                &project.project.id,
                &project.version,
                assets,
                output_dir,
            )
            .await?
        }
        "direct" => {
            forge_publish::direct::package(
                &binary_path,
                &project.project.name,
                &project.version,
                assets,
                output_dir,
            )
            .await?
        }
        "steam" => {
            tracing::info!("Steam publishing not yet implemented (requires Steamworks account)");
            if dry_run {
                tracing::info!("  (dry-run mode — would package for SteamPipe upload)");
            }
            return Ok(());
        }
        "humble" => {
            tracing::info!("Humble publishing not yet implemented");
            return Ok(());
        }
        "xbox" => {
            tracing::info!("Xbox publishing not yet implemented (requires GDK + Partner Center)");
            return Ok(());
        }
        other => {
            anyhow::bail!("unknown publish target: {other}");
        }
    };

    tracing::info!("Package created: {}", package_path.display());

    if dry_run {
        tracing::info!("Dry-run mode — package created but not uploaded");
    } else {
        tracing::info!("Upload not yet implemented for target: {target}");
        tracing::info!("Package is ready at: {}", package_path.display());
    }

    Ok(())
}

/// Find the build output binary for a project.
fn find_build_output(
    project: &ForgeProject,
    _platform: &str,
) -> anyhow::Result<std::path::PathBuf> {
    match &project.engine {
        forge_core::config::EngineConfig::Forge2d { cargo_toml } => {
            // cargo_toml is relative to the forge.toml directory (current dir)
            // e.g. "test-game/Cargo.toml"
            let cargo_toml_path = std::path::Path::new(cargo_toml);
            let project_dir = cargo_toml_path
                .parent()
                .unwrap_or(std::path::Path::new("."));

            // Read the binary name from Cargo.toml
            let contents = std::fs::read_to_string(cargo_toml_path)?;
            let cargo: toml::Value = toml::from_str(&contents)?;
            let binary_name = cargo
                .get("package")
                .and_then(|p| p.get("name"))
                .and_then(|n| n.as_str())
                .ok_or_else(|| anyhow::anyhow!("could not find package name in Cargo.toml"))?;

            // For native builds, binary is in target/release/
            Ok(project_dir.join("target/release").join(binary_name))
        }
        forge_core::config::EngineConfig::Ue5 { .. } => {
            anyhow::bail!("UE5 build output discovery not yet implemented")
        }
    }
}
