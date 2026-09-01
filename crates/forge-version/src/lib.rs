//! forge-version: versioning and lockfile system for Forge Engine.
//!
//! Provides:
//! - Semantic version parsing and bumping
//! - forge.lock lockfile generation (pins engine, mods, assets)
//! - Changelog generation from version history
//! - Dependency compatibility checking

pub mod changelog;
pub mod lockfile;
pub mod semver_bump;

pub use changelog::{Changelog, ChangelogEntry};
pub use lockfile::{AssetPin, EnginePin, Lockfile, ModPin};
pub use semver_bump::{BumpPart, VersionBumper};
