//! Build artifact types.

use std::path::PathBuf;

/// A packaged build artifact ready for publishing.
#[derive(Debug)]
pub struct BuildArtifact {
    /// Path to the packaged binary/app
    pub binary: PathBuf,
    /// Path to the manifest file
    pub manifest: PathBuf,
    /// SHA-256 checksum of the binary
    pub checksum: String,
    /// Platform this artifact was built for
    pub platform: String,
}
