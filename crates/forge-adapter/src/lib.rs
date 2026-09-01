//! forge-adapter: engine abstraction layer.
//!
//! Defines the trait that all engine adapters implement. The build pipeline
//! dispatches to the appropriate adapter based on the engine kind.

use forge_core::target::EngineKind;
use forge_core::ForgeProject;

/// Trait that every engine adapter implements.
#[async_trait::async_trait]
pub trait EngineAdapter: Send + Sync {
    /// Which engine this adapter handles.
    fn engine_kind(&self) -> EngineKind;

    /// Build the game for a target platform.
    async fn build(
        &self,
        project: &ForgeProject,
        platform: &str,
        profile: &str,
    ) -> anyhow::Result<BuildOutput>;
}

/// Output from an engine build.
#[derive(Debug)]
pub struct BuildOutput {
    /// Path to the packaged binary/app
    pub output_path: std::path::PathBuf,
    /// Build metadata
    pub metadata: BuildMetadata,
}

/// Metadata about a completed build.
#[derive(Debug, serde::Serialize)]
pub struct BuildMetadata {
    pub engine: String,
    pub engine_version: String,
    pub platform: String,
    pub profile: String,
    pub build_hash: String,
    pub built_at: String,
}
