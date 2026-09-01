//! Error types for Forge Engine.

use thiserror::Error;

pub type Result<T> = std::result::Result<T, ForgeError>;

#[derive(Debug, Error)]
pub enum ForgeError {
    #[error("configuration error: {0}")]
    Config(String),

    #[error("build error: {0}")]
    Build(String),

    #[error("publish error: {0}")]
    Publish(String),

    #[error("mod error: {0}")]
    Mod(String),

    #[error("adapter error: {0}")]
    Adapter(String),

    #[error("asset error: {0}")]
    Asset(String),

    #[error("version error: {0}")]
    Version(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("unsupported: {0}")]
    Unsupported(String),
}
