//! Build and publish target types.

use serde::{Deserialize, Serialize};

/// Which engine to use for a game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EngineKind {
    /// Unreal Engine 5
    Ue5,
    /// Forge2D (Bevy-based)
    Forge2d,
}

impl EngineKind {
    /// Parse from string (returns None on unknown).
    pub fn parse(s: &str) -> Option<Self> {
        use std::str::FromStr;
        Self::from_str(s).ok()
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ue5 => "ue5",
            Self::Forge2d => "forge2d",
        }
    }
}

impl std::fmt::Display for EngineKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for EngineKind {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "ue5" => Ok(Self::Ue5),
            "forge2d" => Ok(Self::Forge2d),
            _ => Err(format!("unknown engine: {s}")),
        }
    }
}

/// A build target — a combination of engine and platform.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildTarget {
    pub engine: EngineKind,
    pub platform: crate::platform::Platform,
    pub profile: BuildProfile,
}

/// Build profile (debug, release, shipping).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum BuildProfile {
    Debug,
    #[default]
    Release,
    Shipping,
}

impl BuildProfile {
    /// Parse from string (returns None on unknown).
    pub fn parse(s: &str) -> Option<Self> {
        use std::str::FromStr;
        Self::from_str(s).ok()
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Debug => "debug",
            Self::Release => "release",
            Self::Shipping => "shipping",
        }
    }
}

impl std::str::FromStr for BuildProfile {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "debug" => Ok(Self::Debug),
            "release" => Ok(Self::Release),
            "shipping" => Ok(Self::Shipping),
            _ => Err(format!("unknown profile: {s}")),
        }
    }
}

/// A publish target — a store to publish to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PublishTarget {
    /// Linux native (AppImage, .deb, .rpm, Flatpak)
    LinuxNative,
    /// Steam (via SteamPipe)
    Steam,
    /// Humble Bundle / Humble Store
    Humble,
    /// Xbox / Microsoft Store (via GDK)
    Xbox,
    /// itch.io
    Itch,
    /// Direct download (self-hosted)
    Direct,
}

impl PublishTarget {
    /// Parse from string (returns None on unknown).
    pub fn parse(s: &str) -> Option<Self> {
        use std::str::FromStr;
        Self::from_str(s).ok()
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::LinuxNative => "linux-native",
            Self::Steam => "steam",
            Self::Humble => "humble",
            Self::Xbox => "xbox",
            Self::Itch => "itch",
            Self::Direct => "direct",
        }
    }
}

impl std::fmt::Display for PublishTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for PublishTarget {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "linux-native" => Ok(Self::LinuxNative),
            "steam" => Ok(Self::Steam),
            "humble" => Ok(Self::Humble),
            "xbox" => Ok(Self::Xbox),
            "itch" => Ok(Self::Itch),
            "direct" => Ok(Self::Direct),
            _ => Err(format!("unknown publish target: {s}")),
        }
    }
}
