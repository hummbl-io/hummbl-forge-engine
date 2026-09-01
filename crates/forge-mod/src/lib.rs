//! forge-mod: mod system for Forge Engine.
//!
//! Two modes of operation:
//! 1. Moddable-by-design SDK — for games built on Forge Engine
//! 2. Universal mod loader — for existing games (Factorio first)

pub mod loader;
pub mod manifest;
pub mod profiles;
pub mod sdk;

pub use profiles::profile::{GameProfile, InstalledMod, ModInstallResult};
