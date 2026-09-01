//! Store target definitions.
//!
//! Each store is a plugin. The publish pipeline dispatches to the appropriate
//! store plugin based on the publish target.

use forge_core::PublishTarget;

/// Trait that each store plugin implements.
#[async_trait::async_trait]
pub trait Store {
    /// The publish target this store handles.
    fn target(&self) -> PublishTarget;

    /// Package a build artifact for this store (no upload).
    async fn package(&self, artifact_path: &std::path::Path) -> anyhow::Result<std::path::PathBuf>;

    /// Upload a packaged artifact to the store.
    async fn upload(&self, package_path: &std::path::Path) -> anyhow::Result<UploadReceipt>;
}

/// Receipt from a successful upload.
#[derive(Debug, serde::Serialize)]
pub struct UploadReceipt {
    pub store: String,
    pub uploaded_at: String,
    pub build_hash: String,
    pub store_build_id: Option<String>,
    pub status: String,
}
