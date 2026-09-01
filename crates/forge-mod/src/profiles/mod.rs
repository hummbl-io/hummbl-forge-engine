//! Universal mod loader — game profiles for existing games.
//!
//! Each game profile is a plugin that knows how to install, enable, disable,
//! and manage mods for a specific game. The first profile is Factorio.

pub mod factorio;
pub mod profile;

pub use profile::{GameProfile, ModInstallResult};
