//! `forge mod` — mod management commands.

use super::ModActions;

pub async fn run(action: ModActions) -> anyhow::Result<()> {
    match action {
        ModActions::Install { path, game } => {
            tracing::info!("Installing mod from {path}");
            if let Some(g) = game {
                tracing::info!("  Target game: {g}");
            }
        }
        ModActions::List => {
            tracing::info!("Listing installed mods");
        }
        ModActions::Enable { mod_id } => {
            tracing::info!("Enabling mod: {mod_id}");
        }
        ModActions::Disable { mod_id } => {
            tracing::info!("Disabling mod: {mod_id}");
        }
        ModActions::New { name, game } => {
            tracing::info!("Creating new mod scaffold: {name}");
            if let Some(g) = game {
                tracing::info!("  For game: {g}");
            }
        }
    }
    tracing::info!("Mod system not yet implemented — this is a scaffold.");
    Ok(())
}
