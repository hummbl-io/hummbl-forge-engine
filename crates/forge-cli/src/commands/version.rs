//! `forge version` — version management commands.

use std::str::FromStr;

use super::VersionActions;
use forge_core::ForgeProject;
use forge_version::{BumpPart, Changelog, ChangelogEntry, Lockfile, VersionBumper};

pub async fn run(action: VersionActions) -> anyhow::Result<()> {
    match action {
        VersionActions::Show => {
            let project = ForgeProject::from_current_dir()
                .map_err(|e| anyhow::anyhow!("failed to load forge.toml: {e}"))?;
            println!("{} v{}", project.project.name, project.version);
        }
        VersionActions::Bump { part } => {
            bump_version(&part)?;
        }
        VersionActions::Lock => {
            generate_lockfile()?;
        }
        VersionActions::Check => {
            check_lockfile()?;
        }
    }
    Ok(())
}

/// Bump the project version in forge.toml.
fn bump_version(part: &str) -> anyhow::Result<()> {
    let bump_part = BumpPart::from_str(part).map_err(|e| anyhow::anyhow!("{e}"))?;

    let project = ForgeProject::from_current_dir()
        .map_err(|e| anyhow::anyhow!("failed to load forge.toml: {e}"))?;

    let mut bumper = VersionBumper::parse(&project.version)?;
    let old_version = bumper.version_string();
    bumper.bump(bump_part);
    let new_version = bumper.version_string();

    tracing::info!("Bumping version: {old_version} → {new_version}");

    // Update forge.toml
    let forge_toml_path = std::path::Path::new("forge.toml");
    let contents = std::fs::read_to_string(forge_toml_path)?;
    let updated = contents.replace(
        &format!("version = \"{}\"", old_version),
        &format!("version = \"{}\"", new_version),
    );
    std::fs::write(forge_toml_path, updated)?;

    tracing::info!("Updated forge.toml: {old_version} → {new_version}");

    // Add a changelog entry
    let changelog_path = std::path::Path::new(".forge-changelog.json");
    let mut changelog = Changelog::load(changelog_path).unwrap_or(Changelog {
        entries: Vec::new(),
    });
    changelog.add_entry(ChangelogEntry {
        version: new_version.clone(),
        date: today(),
        added: vec![],
        changed: vec![],
        fixed: vec![],
        removed: vec![],
    });
    changelog.save(changelog_path)?;

    println!("Version bumped: {old_version} → {new_version}");
    Ok(())
}

/// Generate a forge.lock lockfile.
fn generate_lockfile() -> anyhow::Result<()> {
    let project = ForgeProject::from_current_dir()
        .map_err(|e| anyhow::anyhow!("failed to load forge.toml: {e}"))?;

    project
        .validate()
        .map_err(|e| anyhow::anyhow!("invalid project: {e}"))?;

    tracing::info!(
        "Generating forge.lock for {} v{}",
        project.project.name,
        project.version
    );

    let lockfile = Lockfile::from_project(&project)?;
    let lockfile_path = std::path::Path::new("forge.lock");
    lockfile.write_to_file(lockfile_path)?;

    tracing::info!(
        "Lockfile written: {} (engine: {}, {} assets pinned)",
        lockfile_path.display(),
        lockfile.engine.kind,
        lockfile.assets.len()
    );

    println!(
        "forge.lock generated ({} assets pinned)",
        lockfile.assets.len()
    );
    Ok(())
}

/// Check if the lockfile is up to date.
fn check_lockfile() -> anyhow::Result<()> {
    let project = ForgeProject::from_current_dir()
        .map_err(|e| anyhow::anyhow!("failed to load forge.toml: {e}"))?;

    let lockfile_path = std::path::Path::new("forge.lock");
    if !lockfile_path.exists() {
        println!("forge.lock not found — run `forge version lock` to generate");
        return Ok(());
    }

    let lockfile = Lockfile::read_from_file(lockfile_path)?;

    if lockfile.is_up_to_date(&project) {
        println!(
            "forge.lock is up to date (v{}, {} assets)",
            lockfile.project_version,
            lockfile.assets.len()
        );
    } else {
        println!("forge.lock is OUT OF DATE");
        println!(
            "  Lockfile: {} v{}",
            lockfile.project_id, lockfile.project_version
        );
        println!("  Project:  {} v{}", project.project.id, project.version);
        println!("Run `forge version lock` to regenerate");
    }
    Ok(())
}

/// Get today's date as YYYY-MM-DD.
fn today() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Simple conversion — not perfectly accurate but good enough for changelog
    let days = now / 86400;
    let year = 1970 + (days / 365);
    let day_of_year = days % 365;
    let month = (day_of_year / 30).min(11) + 1;
    let day = (day_of_year % 30) + 1;
    format!("{year:04}-{month:02}-{day:02}")
}
