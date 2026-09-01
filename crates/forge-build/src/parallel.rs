//! Multi-target parallel build orchestration.
//!
//! Builds for multiple platforms simultaneously using async tasks.

use forge_core::ForgeProject;

/// A multi-target build request.
#[derive(Debug, Clone)]
pub struct ParallelBuildRequest {
    pub project: ForgeProject,
    pub targets: Vec<BuildTarget>,
    pub profile: String,
}

/// A single build target within a parallel build.
#[derive(Debug, Clone)]
pub struct BuildTarget {
    pub platform: String,
    pub profile: String,
}

/// Result of a parallel build.
#[derive(Debug)]
pub struct ParallelBuildResult {
    pub results: Vec<TargetBuildResult>,
    pub total_duration_secs: f64,
}

/// Result of building a single target.
#[derive(Debug)]
pub struct TargetBuildResult {
    pub platform: String,
    pub success: bool,
    pub output_path: Option<std::path::PathBuf>,
    pub error: Option<String>,
    pub duration_secs: f64,
}

/// Run a multi-target parallel build.
pub async fn build_parallel(
    project: &ForgeProject,
    targets: Vec<(String, String)>,
) -> anyhow::Result<ParallelBuildResult> {
    let start = std::time::Instant::now();

    tracing::info!(
        "Starting parallel build: {} targets for {}",
        targets.len(),
        project.project.name
    );

    let mut tasks = Vec::new();

    for (platform, profile) in targets {
        let task = tokio::spawn(build_single_target(
            project.clone(),
            platform.clone(),
            profile,
        ));
        tasks.push(task);
    }

    let mut results = Vec::new();
    for task in tasks {
        match task.await {
            Ok(result) => results.push(result),
            Err(e) => results.push(TargetBuildResult {
                platform: "unknown".to_string(),
                success: false,
                output_path: None,
                error: Some(format!("task panicked: {e}")),
                duration_secs: 0.0,
            }),
        }
    }

    let duration = start.elapsed().as_secs_f64();

    let succeeded = results.iter().filter(|r| r.success).count();
    let failed = results.len() - succeeded;

    tracing::info!(
        "Parallel build complete: {succeeded} succeeded, {failed} failed in {duration:.1}s"
    );

    Ok(ParallelBuildResult {
        results,
        total_duration_secs: duration,
    })
}

/// Build a single target (runs in a spawned task).
async fn build_single_target(
    project: ForgeProject,
    platform: String,
    profile: String,
) -> TargetBuildResult {
    let start = std::time::Instant::now();
    let platform_clone = platform.clone();

    tracing::info!("Building for {platform_clone} ({profile})...");

    match crate::pipeline::build(&project, &platform, &profile).await {
        Ok(result) => {
            let duration = start.elapsed().as_secs_f64();
            tracing::info!("Built {platform_clone} in {duration:.1}s");
            TargetBuildResult {
                platform: platform_clone,
                success: true,
                output_path: Some(result.output_path),
                error: None,
                duration_secs: duration,
            }
        }
        Err(e) => {
            let duration = start.elapsed().as_secs_f64();
            tracing::error!("Failed to build {platform_clone}: {e}");
            TargetBuildResult {
                platform: platform_clone,
                success: false,
                output_path: None,
                error: Some(e.to_string()),
                duration_secs: duration,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_target_construction() {
        let target = BuildTarget {
            platform: "linux-native".to_string(),
            profile: "release".to_string(),
        };
        assert_eq!(target.platform, "linux-native");
        assert_eq!(target.profile, "release");
    }
}
