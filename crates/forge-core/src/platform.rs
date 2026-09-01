//! Platform definitions for Forge Engine.

use serde::{Deserialize, Serialize};

/// A target platform for game builds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Platform {
    /// Linux x86_64 (native, no store)
    LinuxNative,

    /// Linux x86_64 via Steam
    LinuxSteam,

    /// Windows x86_64 (native, no store)
    WindowsNative,

    /// Windows x86_64 via Steam
    WindowsSteam,

    /// Xbox/Microsoft (via GDK)
    Xbox,

    /// Web (WebAssembly)
    Web,

    /// macOS (Apple Silicon)
    MacosArm,

    /// macOS (Intel)
    MacosX86,
}

impl Platform {
    /// Parse a platform string from forge.toml into a Platform enum.
    pub fn parse(s: &str) -> Option<Self> {
        use std::str::FromStr;
        Self::from_str(s).ok()
    }

    /// Get the string identifier used in forge.toml
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::LinuxNative => "linux-native",
            Self::LinuxSteam => "linux-steam",
            Self::WindowsNative => "windows-native",
            Self::WindowsSteam => "windows-steam",
            Self::Xbox => "xbox",
            Self::Web => "web",
            Self::MacosArm => "macos-arm",
            Self::MacosX86 => "macos-x86",
        }
    }

    /// Get the operating system family
    pub fn os(&self) -> &'static str {
        match self {
            Self::LinuxNative | Self::LinuxSteam => "linux",
            Self::WindowsNative | Self::WindowsSteam | Self::Xbox => "windows",
            Self::Web => "web",
            Self::MacosArm | Self::MacosX86 => "macos",
        }
    }

    /// Whether this platform targets a store
    pub fn is_store(&self) -> bool {
        matches!(self, Self::LinuxSteam | Self::WindowsSteam | Self::Xbox)
    }
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for Platform {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "linux-native" => Ok(Self::LinuxNative),
            "linux-steam" => Ok(Self::LinuxSteam),
            "windows-native" => Ok(Self::WindowsNative),
            "windows-steam" => Ok(Self::WindowsSteam),
            "xbox" => Ok(Self::Xbox),
            "web" => Ok(Self::Web),
            "macos-arm" => Ok(Self::MacosArm),
            "macos-x86" => Ok(Self::MacosX86),
            _ => Err(format!("unknown platform: {s}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_roundtrip() {
        for p in [
            Platform::LinuxNative,
            Platform::LinuxSteam,
            Platform::WindowsSteam,
            Platform::Xbox,
            Platform::Web,
        ] {
            let s = p.as_str();
            assert_eq!(Platform::parse(s), Some(p));
        }
    }

    #[test]
    fn test_unknown_platform() {
        assert_eq!(Platform::parse("switch"), None);
    }

    #[test]
    fn test_is_store() {
        assert!(Platform::LinuxSteam.is_store());
        assert!(!Platform::LinuxNative.is_store());
    }
}
