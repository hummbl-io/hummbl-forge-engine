//! `forge version` — version management commands.

use super::VersionActions;

pub async fn run(action: VersionActions) -> anyhow::Result<()> {
    match action {
        VersionActions::Show => {
            let project = forge_core::ForgeProject::from_current_dir()
                .map_err(|e| anyhow::anyhow!("failed to load forge.toml: {e}"))?;
            println!("{} v{}", project.project.name, project.version);
        }
        VersionActions::Bump { part } => {
            tracing::info!("Bumping version: {part}");
        }
        VersionActions::Lock => {
            tracing::info!("Generating lockfile");
        }
        VersionActions::Check => {
            tracing::info!("Checking version compatibility");
        }
    }
    Ok(())
}
