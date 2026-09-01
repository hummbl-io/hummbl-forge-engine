//! Forge2D adapter — builds Forge2D (Bevy-based) games.
//!
//! Forge2D games are Rust projects using Bevy. The adapter invokes
//! cargo to build them for the target platform.

use forge_adapter::{BuildOutput, EngineAdapter};
use forge_core::config::EngineConfig;
use forge_core::target::EngineKind;
use forge_core::ForgeProject;

/// Forge2D engine adapter.
pub struct Forge2dAdapter;

impl Forge2dAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Forge2dAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl EngineAdapter for Forge2dAdapter {
    fn engine_kind(&self) -> EngineKind {
        EngineKind::Forge2d
    }

    async fn build(
        &self,
        project: &ForgeProject,
        platform: &str,
        profile: &str,
    ) -> anyhow::Result<BuildOutput> {
        let cargo_toml = match &project.engine {
            EngineConfig::Forge2d { cargo_toml } => cargo_toml,
            _ => anyhow::bail!("project is not a Forge2D project"),
        };

        tracing::info!("Forge2D build: {cargo_toml} for {platform} ({profile})");

        // TODO: invoke cargo build with appropriate target
        // For Linux: cargo build --release
        // For Web: cargo build --target wasm32-unknown-unknown
        anyhow::bail!("Forge2D build not yet implemented — cargo invocation pending")
    }
}
