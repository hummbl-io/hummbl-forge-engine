//! Mod SDK — tools for mod authors and the runtime for mod execution.
//!
//! Provides:
//! - Mod API version tracking
//! - Lua scripting runtime (via rlua)
//! - Permission sandboxing
//! - Dependency resolution
//! - Hot-reload support
//! - Mod context and lifecycle

pub mod api;
pub mod deps;
pub mod hot_reload;
pub mod sandbox;

pub use api::{ModApiVersion, ModContext, ModEvent, ModResult};
pub use deps::{DependencyResolver, ResolutionResult};
pub use hot_reload::HotReloader;
pub use sandbox::{PermissionChecker, SandboxConfig, SandboxError};

/// The mod API version.
pub const MOD_API_VERSION: &str = "0.1.0";
