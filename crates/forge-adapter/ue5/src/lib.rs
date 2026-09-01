//! UE5 adapter — communicates with Unreal Engine 5 via UAT/UBT subprocess.
//!
//! Uses Unreal Automation Tool (UAT) and UnrealBuildTool (UBT) to build
//! UE5 projects programmatically.

use forge_adapter::{BuildOutput, EngineAdapter};
use forge_core::config::EngineConfig;
use forge_core::target::EngineKind;
use forge_core::ForgeProject;

/// UE5 engine adapter.
pub struct Ue5Adapter {
    /// Path to UE5 installation (Engine directory)
    engine_path: std::path::PathBuf,
}

impl Ue5Adapter {
    /// Create a new UE5 adapter from an engine path.
    pub fn new(engine_path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            engine_path: engine_path.into(),
        }
    }

    /// Try to auto-detect UE5 installation.
    pub fn auto_detect() -> Option<Self> {
        // Common UE5 installation paths
        let candidates = [
            "C:\\Program Files\\Epic Games\\UE_5.4\\Engine",
            "C:\\Program Files\\Epic Games\\UE_5.5\\Engine",
            "/home/reuben/UE_5.4/Engine",
            "/opt/UE_5.4/Engine",
        ];

        for candidate in &candidates {
            let path = std::path::Path::new(candidate);
            if path.exists() {
                return Some(Self::new(path));
            }
        }

        None
    }

    /// Get the path to the Unreal Automation Tool (UAT) script.
    fn uat_path(&self) -> std::path::PathBuf {
        // UAT is at Engine/Build/BatchFiles/RunUAT.bat (Windows)
        // or Engine/Build/BatchFiles/RunUAT.sh (Linux)
        if cfg!(windows) {
            self.engine_path.join("Build/BatchFiles/RunUAT.bat")
        } else {
            self.engine_path.join("Build/BatchFiles/RunUAT.sh")
        }
    }
}

#[async_trait::async_trait]
impl EngineAdapter for Ue5Adapter {
    fn engine_kind(&self) -> EngineKind {
        EngineKind::Ue5
    }

    async fn build(
        &self,
        project: &ForgeProject,
        platform: &str,
        profile: &str,
    ) -> anyhow::Result<BuildOutput> {
        let uproject = match &project.engine {
            EngineConfig::Ue5 { uproject, .. } => uproject,
            _ => anyhow::bail!("project is not a UE5 project"),
        };

        let uat = self.uat_path();
        tracing::info!(
            "UE5 build: {uproject} for {platform} ({profile}) via UAT at {}",
            uat.display()
        );

        // TODO: invoke UAT with BuildPlugin or BuildCookRun
        // RunUAT BuildCookRun -project=path/to/Game.uproject
        //   -platform=Linux -clientconfig=Shipping -cook -build -stage -pak -archive
        anyhow::bail!("UE5 build not yet implemented — UAT invocation pending")
    }
}
