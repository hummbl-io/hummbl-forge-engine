//! `forge mod` — mod management commands.
//!
//! Supports two modes:
//! 1. Forge-native mods (for games built on Forge Engine) — uses mod.toml
//! 2. Universal mod loader (for existing games) — uses game profiles

use super::ModActions;
use forge_mod::GameProfile;

pub async fn run(action: ModActions) -> anyhow::Result<()> {
    match action {
        ModActions::Install { path, game } => {
            install_mod(&path, game.as_deref())?;
        }
        ModActions::List => {
            list_mods()?;
        }
        ModActions::Enable { mod_id } => {
            enable_mod(&mod_id)?;
        }
        ModActions::Disable { mod_id } => {
            disable_mod(&mod_id)?;
        }
        ModActions::New { name, game } => {
            create_mod_scaffold(&name, game.as_deref())?;
        }
    }
    Ok(())
}

/// Install a mod for a specific game (universal mod loader).
fn install_mod(mod_path: &str, game: Option<&str>) -> anyhow::Result<()> {
    let game = game.unwrap_or("factorio"); // default to Factorio for now
    let mod_path = std::path::Path::new(mod_path);

    if !mod_path.exists() {
        anyhow::bail!("mod file not found: {}", mod_path.display());
    }

    match game {
        "factorio" => {
            let profile = forge_mod::profiles::factorio::FactorioProfile::new();
            let install_path = profile.detect_install_path().ok_or_else(|| {
                anyhow::anyhow!("Factorio installation not detected — set mods dir manually")
            })?;

            tracing::info!("Installing mod to: {}", install_path.display());
            let result = profile.install_mod(mod_path, &install_path)?;
            tracing::info!(
                "  ✓ Installed: {} v{} → {}",
                result.mod_id,
                result.mod_version,
                result.installed_path.display()
            );
        }
        other => {
            anyhow::bail!("unsupported game profile: {other} (only 'factorio' is implemented)");
        }
    }
    Ok(())
}

/// List installed mods for the default game.
fn list_mods() -> anyhow::Result<()> {
    let profile = forge_mod::profiles::factorio::FactorioProfile::new();
    let install_path = profile
        .detect_install_path()
        .ok_or_else(|| anyhow::anyhow!("Factorio installation not detected"))?;

    let mods = profile.list_mods(&install_path)?;
    if mods.is_empty() {
        tracing::info!("No mods installed.");
    } else {
        tracing::info!("Installed mods ({}):", mods.len());
        for m in &mods {
            let status = if m.enabled { "enabled" } else { "disabled" };
            println!("  {status:8} {} v{}", m.id, m.version);
        }
    }
    Ok(())
}

/// Enable a mod by ID.
fn enable_mod(mod_id: &str) -> anyhow::Result<()> {
    let profile = forge_mod::profiles::factorio::FactorioProfile::new();
    let install_path = profile
        .detect_install_path()
        .ok_or_else(|| anyhow::anyhow!("Factorio installation not detected"))?;

    profile.enable_mod(mod_id, &install_path)?;
    tracing::info!("Enabled: {mod_id}");
    Ok(())
}

/// Disable a mod by ID.
fn disable_mod(mod_id: &str) -> anyhow::Result<()> {
    let profile = forge_mod::profiles::factorio::FactorioProfile::new();
    let install_path = profile
        .detect_install_path()
        .ok_or_else(|| anyhow::anyhow!("Factorio installation not detected"))?;

    profile.disable_mod(mod_id, &install_path)?;
    tracing::info!("Disabled: {mod_id}");
    Ok(())
}

/// Create a new mod scaffold.
fn create_mod_scaffold(name: &str, game: Option<&str>) -> anyhow::Result<()> {
    let game = game.unwrap_or("factorio");
    tracing::info!("Creating mod scaffold: {name} for {game}");

    let mod_dir = std::path::Path::new(name);
    if mod_dir.exists() {
        anyhow::bail!("directory already exists: {name}");
    }

    match game {
        "factorio" => {
            std::fs::create_dir_all(mod_dir)?;

            // Create info.json (Factorio mod metadata)
            let info_json = format!(
                r#"{{
  "name": "{name}",
  "version": "0.1.0",
  "title": "{name}",
  "author": "Your Name",
  "factorio_version": "2.0",
  "dependencies": []
}}
"#
            );
            std::fs::write(mod_dir.join("info.json"), info_json)?;

            // Create a placeholder data.lua
            std::fs::write(
                mod_dir.join("data.lua"),
                "-- Mod data definitions go here\n",
            )?;

            // Create a placeholder control.lua
            std::fs::write(
                mod_dir.join("control.lua"),
                "-- Mod control/script logic goes here\n",
            )?;

            tracing::info!("  Created: {name}/info.json");
            tracing::info!("  Created: {name}/data.lua");
            tracing::info!("  Created: {name}/control.lua");
            tracing::info!(
                "Mod scaffold created. Edit info.json and the Lua files to build your mod."
            );
        }
        other => {
            anyhow::bail!("unsupported game profile: {other}");
        }
    }
    Ok(())
}
