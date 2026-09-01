//! `forge check` — validate project configuration.

pub async fn run() -> anyhow::Result<()> {
    let project = forge_core::ForgeProject::from_current_dir()
        .map_err(|e| anyhow::anyhow!("failed to load forge.toml: {e}"))?;

    match project.validate() {
        Ok(()) => {
            println!("OK — forge.toml is valid");
            println!("  Project: {} v{}", project.project.name, project.version);
            Ok(())
        }
        Err(e) => {
            anyhow::bail!("INVALID — {e}");
        }
    }
}
