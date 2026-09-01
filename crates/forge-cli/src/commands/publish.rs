//! `forge publish` — publish a build to a store.

pub async fn run(target: &str, dry_run: bool) -> anyhow::Result<()> {
    tracing::info!("Publishing to {target} (dry_run: {dry_run})");
    tracing::info!("Publish pipeline not yet implemented — this is a scaffold.");
    Ok(())
}
