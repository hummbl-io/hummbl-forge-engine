//! Universal mod loader — game profiles for existing games.
//!
//! Each game profile is a plugin that knows how to install, enable, disable,
//! and manage mods for a specific game.

pub mod factorio;
pub mod generic;
pub mod minecraft;
pub mod profile;
pub mod skyrim;

pub use profile::{GameProfile, ModInstallResult};
