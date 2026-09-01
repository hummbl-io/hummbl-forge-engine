//! forge-core: shared types, configuration, and error handling for Forge Engine.

pub mod config;
pub mod error;
pub mod platform;
pub mod target;

pub use config::ForgeProject;
pub use error::{ForgeError, Result};
pub use platform::Platform;
pub use target::{BuildTarget, EngineKind, PublishTarget};
