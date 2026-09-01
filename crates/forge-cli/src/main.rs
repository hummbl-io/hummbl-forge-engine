//! forge-cli: the `forge` binary — entry point for the Forge Engine CLI.

mod commands;

use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

/// HUMMBL Forge Engine — meta-engine layer for building, publishing,
/// modding, and versioning games.
#[derive(Parser)]
#[command(name = "forge", version, about, long_about = None)]
struct Cli {
    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new Forge Engine game project
    Init {
        /// Project name
        name: String,

        /// Engine to use (ue5, forge2d)
        #[arg(long, default_value = "forge2d")]
        engine: String,
    },

    /// Build a game for one or more platforms
    Build {
        /// Engine to use (overrides forge.toml)
        #[arg(long)]
        engine: Option<String>,

        /// Platform to build for (overrides forge.toml)
        #[arg(long)]
        platform: Option<String>,

        /// Build profile (debug, release, shipping)
        #[arg(long, default_value = "release")]
        profile: String,
    },

    /// Publish a build to a store
    Publish {
        /// Target store (steam, humble, xbox, linux-native, itch, direct)
        #[arg(long)]
        target: String,

        /// Dry-run mode (build package without uploading)
        #[arg(long)]
        dry_run: bool,
    },

    /// Mod management
    Mod {
        #[command(subcommand)]
        action: ModActions,
    },

    /// Version management
    Version {
        #[command(subcommand)]
        action: VersionActions,
    },

    /// Show project info
    Info,

    /// Check project configuration
    Check,
}

#[derive(Subcommand)]
pub enum ModActions {
    /// Install a mod
    Install {
        /// Path to mod directory or archive
        path: String,

        /// Target game (for universal mod loader)
        #[arg(long)]
        game: Option<String>,
    },

    /// List installed mods
    List,

    /// Enable a mod
    Enable {
        /// Mod ID
        mod_id: String,
    },

    /// Disable a mod
    Disable {
        /// Mod ID
        mod_id: String,
    },

    /// Create a new mod scaffold
    New {
        /// Mod name
        name: String,

        /// Target game project
        #[arg(long)]
        game: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum VersionActions {
    /// Show current version
    Show,

    /// Bump version
    Bump {
        /// Bump type (major, minor, patch)
        part: String,
    },

    /// Generate lockfile
    Lock,

    /// Check version compatibility
    Check,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let filter = if cli.verbose {
        EnvFilter::new("debug")
    } else {
        EnvFilter::new("info")
    };
    tracing_subscriber::fmt().with_env_filter(filter).init();

    match cli.command {
        Commands::Init { name, engine } => commands::init::run(&name, &engine).await,
        Commands::Build {
            engine,
            platform,
            profile,
        } => commands::build::run(engine.as_deref(), platform.as_deref(), &profile).await,
        Commands::Publish { target, dry_run } => commands::publish::run(&target, dry_run).await,
        Commands::Mod { action } => commands::mod_cmd::run(action).await,
        Commands::Version { action } => commands::version::run(action).await,
        Commands::Info => commands::info::run().await,
        Commands::Check => commands::check::run().await,
    }
}
