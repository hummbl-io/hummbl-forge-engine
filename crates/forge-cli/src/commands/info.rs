//! `forge info` — show project information.

pub async fn run() -> anyhow::Result<()> {
    let project = forge_core::ForgeProject::from_current_dir()
        .map_err(|e| anyhow::anyhow!("failed to load forge.toml: {e}"))?;

    println!("Project:     {}", project.project.name);
    println!("ID:          {}", project.project.id);
    println!("Version:     {}", project.version);
    println!("Description: {}", project.project.description);
    println!("Authors:     {}", project.project.authors.join(", "));
    println!();

    match &project.engine {
        forge_core::config::EngineConfig::Ue5 {
            uproject,
            version,
            engine_path,
        } => {
            println!("Engine:      UE5 {version}");
            println!("  Project:   {uproject}");
            if let Some(p) = engine_path {
                println!("  Path:      {p}");
            }
        }
        forge_core::config::EngineConfig::Forge2d { cargo_toml } => {
            println!("Engine:      Forge2D (Bevy)");
            println!("  Cargo:     {cargo_toml}");
        }
    }

    println!();
    println!("Build targets:   {}", project.build.targets.join(", "));
    println!("Publish targets: {}", project.publish.targets.join(", "));
    println!("Mods enabled:    {}", project.mods.enabled);
    println!("Dry-run:         {}", project.publish.dry_run);

    Ok(())
}
