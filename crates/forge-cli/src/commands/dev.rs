//! `forge dev` — development server with watch + hot-reload.
//!
//! Watches source files and automatically rebuilds + restarts the game
//! when changes are detected.

use std::process::Command;
use std::time::{Duration, SystemTime};

/// Run the development server.
pub async fn run(watch: bool) -> anyhow::Result<()> {
    let project = forge_core::ForgeProject::from_current_dir()
        .map_err(|e| anyhow::anyhow!("failed to load forge.toml: {e}"))?;

    tracing::info!(
        "Starting dev server for {} v{}",
        project.project.name,
        project.version
    );

    if !watch {
        // One-shot build and run
        return build_and_run(&project).await;
    }

    // Watch mode: poll for changes, rebuild on change
    let mut watcher = DevWatcher::new();
    watcher.add_watch("src");
    watcher.add_watch("assets");

    // Initial build
    build_and_run(&project).await?;

    tracing::info!("Watching for changes (Ctrl+C to stop)...");

    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;

        if let Some(changed) = watcher.check_changes() {
            tracing::info!("Change detected in: {}", changed);
            tracing::info!("Rebuilding...");
            if let Err(e) = build_and_run(&project).await {
                tracing::error!("Rebuild failed: {e}");
            }
        }
    }
}

/// Build and run the game once.
async fn build_and_run(project: &forge_core::ForgeProject) -> anyhow::Result<()> {
    // Build using the build pipeline
    let result = forge_build::pipeline::build(project, "linux-native", "debug")
        .await
        .map_err(|e| anyhow::anyhow!("build failed: {e}"))?;

    tracing::info!("Build complete: {}", result.output_path.display());

    // Run the game
    tracing::info!("Running game...");
    let status = Command::new(&result.output_path).status()?;

    if !status.success() {
        tracing::warn!("Game exited with status: {status}");
    }

    Ok(())
}

/// File system watcher for development.
struct DevWatcher {
    watches: Vec<(String, SystemTime)>,
}

impl DevWatcher {
    fn new() -> Self {
        Self {
            watches: Vec::new(),
        }
    }

    fn add_watch(&mut self, path: &str) {
        let p = std::path::Path::new(path);
        if p.exists() {
            let modified = get_dir_mtime(p).unwrap_or(SystemTime::UNIX_EPOCH);
            self.watches.push((path.to_string(), modified));
            tracing::debug!("Watching: {path}");
        }
    }

    fn check_changes(&mut self) -> Option<String> {
        let mut changed = None;
        let mut updates = Vec::new();

        for (path, last_modified) in &self.watches {
            let p = std::path::Path::new(path);
            if let Some(current) = get_dir_mtime(p) {
                if current != *last_modified {
                    changed = Some(path.clone());
                    updates.push((path.clone(), current));
                }
            }
        }

        for (path, mtime) in updates {
            if let Some(entry) = self.watches.iter_mut().find(|(p, _)| p == &path) {
                entry.1 = mtime;
            }
        }

        changed
    }
}

/// Get the most recent modification time in a directory.
fn get_dir_mtime(path: &std::path::Path) -> Option<SystemTime> {
    if !path.is_dir() {
        return None;
    }

    let mut latest: Option<SystemTime> = None;

    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_dir() {
                if let Some(sub_latest) = get_dir_mtime(&entry_path) {
                    latest = Some(match latest {
                        Some(curr) if curr > sub_latest => curr,
                        _ => sub_latest,
                    });
                }
            } else if let Ok(metadata) = entry.metadata() {
                if let Ok(modified) = metadata.modified() {
                    latest = Some(match latest {
                        Some(curr) if curr > modified => curr,
                        _ => modified,
                    });
                }
            }
        }
    }

    latest
}
