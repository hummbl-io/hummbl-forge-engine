//! Forge2D adapter — builds Forge2D (Bevy-based) games.
//!
//! Forge2D games are Rust projects using Bevy. The adapter invokes
//! cargo to build them for the target platform.

use std::path::PathBuf;
use std::process::Command;

use forge_adapter::{BuildMetadata, BuildOutput, EngineAdapter};
use forge_core::config::EngineConfig;
use forge_core::target::EngineKind;
use forge_core::ForgeProject;

/// Forge2D engine adapter.
pub struct Forge2dAdapter {
    /// The directory containing the game's Cargo.toml
    /// (resolved from the forge.toml's engine.cargo_toml path)
    project_dir: PathBuf,
}

impl Forge2dAdapter {
    /// Create a new Forge2D adapter from a project directory.
    pub fn new(project_dir: impl Into<PathBuf>) -> Self {
        Self {
            project_dir: project_dir.into(),
        }
    }

    /// Resolve the cargo target triple for a platform.
    fn target_triple(platform: &str) -> Option<&'static str> {
        match platform {
            "linux-native" | "linux-steam" => None, // native build, no --target needed
            "windows-native" | "windows-steam" => Some("x86_64-pc-windows-msvc"),
            "web" => Some("wasm32-unknown-unknown"),
            "macos-arm" => Some("aarch64-apple-darwin"),
            "macos-x86" => Some("x86_64-apple-darwin"),
            _ => None,
        }
    }

    /// Determine the output binary name and path after cargo build.
    fn output_binary_path(&self, platform: &str, profile: &str) -> PathBuf {
        let target_dir = self.project_dir.join("target");

        // If a specific target triple is used, cargo puts output in target/<triple>/
        let target_subdir = match Self::target_triple(platform) {
            Some(triple) => target_dir.join(triple),
            None => target_dir,
        };

        let profile_dir = match profile {
            "debug" => "debug",
            _ => "release",
        };

        target_subdir.join(profile_dir)
    }

    /// Build the cargo command for the target platform and profile.
    fn build_cargo_command(&self, platform: &str, profile: &str) -> Command {
        let mut cmd = Command::new("cargo");
        cmd.arg("build");
        cmd.current_dir(&self.project_dir);

        // Profile
        match profile {
            "debug" => {} // default
            "shipping" => {
                cmd.arg("--release");
                // Shipping adds extra optimization flags
                cmd.env("RUSTFLAGS", "-C opt-level=3 -C lto=fat -C codegen-units=1");
            }
            _ => {
                cmd.arg("--release");
            }
        }

        // Target triple (for cross-compilation)
        if let Some(triple) = Self::target_triple(platform) {
            cmd.arg("--target").arg(triple);
        }

        cmd
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

        // Verify Cargo.toml exists
        // cargo_toml is relative to the forge.toml directory, but project_dir
        // is the parent of Cargo.toml. So the file is project_dir/Cargo.toml
        // (the filename from cargo_toml, not the full relative path)
        let cargo_toml_filename = std::path::Path::new(cargo_toml)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Cargo.toml");
        let cargo_toml_path = self.project_dir.join(cargo_toml_filename);
        if !cargo_toml_path.exists() {
            anyhow::bail!("Cargo.toml not found at {}", cargo_toml_path.display());
        }

        // Build the cargo command
        let mut cmd = self.build_cargo_command(platform, profile);
        tracing::info!("Running: {:?}", cmd);

        // Execute
        let output = cmd
            .output()
            .map_err(|e| anyhow::anyhow!("failed to run cargo: {e}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("cargo build failed:\n{stderr}")
        }

        // Find the output binary
        // Cargo names the binary after the package name in Cargo.toml
        let binary_name = extract_binary_name(&cargo_toml_path)?;
        let output_dir = self.output_binary_path(platform, profile);
        let binary_path = output_dir.join(&binary_name);

        if !binary_path.exists() {
            // On Windows, it would be .exe
            let exe_path = output_dir.join(format!("{binary_name}.exe"));
            if exe_path.exists() {
                return Ok(BuildOutput {
                    output_path: exe_path,
                    metadata: BuildMetadata {
                        engine: "forge2d".to_string(),
                        engine_version: "0.1.0".to_string(),
                        platform: platform.to_string(),
                        profile: profile.to_string(),
                        build_hash: compute_dir_hash(&output_dir)?,
                        built_at: current_timestamp(),
                    },
                });
            }
            anyhow::bail!(
                "build succeeded but binary not found at {} (expected {} or {}.exe)",
                output_dir.display(),
                binary_name,
                binary_name
            );
        }

        Ok(BuildOutput {
            output_path: binary_path.clone(),
            metadata: BuildMetadata {
                engine: "forge2d".to_string(),
                engine_version: "0.1.0".to_string(),
                platform: platform.to_string(),
                profile: profile.to_string(),
                build_hash: compute_file_hash(&binary_path)?,
                built_at: current_timestamp(),
            },
        })
    }
}

/// Extract the binary name from a Cargo.toml file.
fn extract_binary_name(cargo_toml_path: &std::path::Path) -> anyhow::Result<String> {
    let contents = std::fs::read_to_string(cargo_toml_path)
        .map_err(|e| anyhow::anyhow!("failed to read Cargo.toml: {e}"))?;
    let cargo_toml: toml::Value = toml::from_str(&contents)
        .map_err(|e| anyhow::anyhow!("failed to parse Cargo.toml: {e}"))?;

    // Check for [[bin]] name first
    if let Some(bins) = cargo_toml.get("bin").and_then(|v| v.as_array()) {
        if let Some(first_bin) = bins.first() {
            if let Some(name) = first_bin.get("name").and_then(|v| v.as_str()) {
                return Ok(name.to_string());
            }
        }
    }

    // Fall back to package name
    if let Some(name) = cargo_toml
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
    {
        return Ok(name.to_string());
    }

    anyhow::bail!("could not determine binary name from Cargo.toml")
}

/// Compute SHA-256 hash of a file.
fn compute_file_hash(path: &std::path::Path) -> anyhow::Result<String> {
    use sha2::{Digest, Sha256};
    let data =
        std::fs::read(path).map_err(|e| anyhow::anyhow!("failed to read file for hashing: {e}"))?;
    let mut hasher = Sha256::new();
    hasher.update(&data);
    Ok(hex::encode(hasher.finalize()))
}

/// Compute a hash representing the build output directory.
fn compute_dir_hash(dir: &std::path::Path) -> anyhow::Result<String> {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Ok(data) = std::fs::read(&path) {
                    hasher.update(&data);
                }
            }
        }
    }
    Ok(hex::encode(hasher.finalize()))
}

/// Get current timestamp in ISO 8601 format.
fn current_timestamp() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{now}")
}

// Add sha2 and hex deps for hashing
extern crate hex;
extern crate sha2;
