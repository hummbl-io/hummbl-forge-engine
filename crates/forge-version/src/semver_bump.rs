//! Semantic version bumping.
//!
//! Parses version strings and bumps them according to semver rules.

use std::fmt;

/// Which part of a semantic version to bump.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BumpPart {
    Major,
    Minor,
    Patch,
    PreRelease,
}

impl fmt::Display for BumpPart {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Major => write!(f, "major"),
            Self::Minor => write!(f, "minor"),
            Self::Patch => write!(f, "patch"),
            Self::PreRelease => write!(f, "pre-release"),
        }
    }
}

impl std::str::FromStr for BumpPart {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "major" => Ok(Self::Major),
            "minor" => Ok(Self::Minor),
            "patch" => Ok(Self::Patch),
            "pre" | "prerelease" | "pre-release" => Ok(Self::PreRelease),
            _ => Err(format!(
                "unknown bump part: {s} (expected major, minor, patch, or pre)"
            )),
        }
    }
}

/// Version bumper — parses, bumps, and formats semantic versions.
pub struct VersionBumper {
    version: semver::Version,
}

impl VersionBumper {
    /// Parse a version string.
    pub fn parse(version: &str) -> anyhow::Result<Self> {
        let version = semver::Version::parse(version)
            .map_err(|e| anyhow::anyhow!("invalid version '{version}': {e}"))?;
        Ok(Self { version })
    }

    /// Get the current version.
    pub fn current(&self) -> &semver::Version {
        &self.version
    }

    /// Bump the version by the specified part.
    pub fn bump(&mut self, part: BumpPart) -> &semver::Version {
        match part {
            BumpPart::Major => {
                self.version.major += 1;
                self.version.minor = 0;
                self.version.patch = 0;
                self.version.pre = semver::Prerelease::EMPTY;
            }
            BumpPart::Minor => {
                self.version.minor += 1;
                self.version.patch = 0;
                self.version.pre = semver::Prerelease::EMPTY;
            }
            BumpPart::Patch => {
                if !self.version.pre.is_empty() {
                    // If it's a pre-release, just strip the pre-release tag
                    self.version.pre = semver::Prerelease::EMPTY;
                } else {
                    self.version.patch += 1;
                }
            }
            BumpPart::PreRelease => {
                self.bump_pre_release();
            }
        }
        &self.version
    }

    /// Bump the pre-release component.
    fn bump_pre_release(&mut self) {
        let current_pre = self.version.pre.as_str();
        if current_pre.is_empty() {
            // No pre-release — start with alpha.0
            self.version.pre = semver::Prerelease::new("alpha.0").unwrap();
        } else {
            // Parse "alpha.0", "beta.1", "rc.2" etc.
            if let Some(new_pre) = bump_pre_release_string(current_pre) {
                self.version.pre = semver::Prerelease::new(&new_pre).unwrap();
            }
        }
    }

    /// Format the version as a string.
    pub fn version_string(&self) -> String {
        self.version.to_string()
    }
}

/// Bump a pre-release string like "alpha.0" → "alpha.1".
fn bump_pre_release_string(pre: &str) -> Option<String> {
    // Split on the last '.'
    if let Some(pos) = pre.rfind('.') {
        let prefix = &pre[..pos];
        let suffix = &pre[pos + 1..];
        if let Ok(num) = suffix.parse::<u32>() {
            return Some(format!("{}.{}", prefix, num + 1));
        }
    }
    // If no numeric suffix, append .1
    Some(format!("{pre}.1"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let b = VersionBumper::parse("1.2.3").unwrap();
        assert_eq!(b.version_string(), "1.2.3");
    }

    #[test]
    fn test_parse_with_pre() {
        let b = VersionBumper::parse("1.0.0-alpha.1").unwrap();
        assert_eq!(b.version_string(), "1.0.0-alpha.1");
    }

    #[test]
    fn test_bump_major() {
        let mut b = VersionBumper::parse("1.2.3").unwrap();
        b.bump(BumpPart::Major);
        assert_eq!(b.version_string(), "2.0.0");
    }

    #[test]
    fn test_bump_minor() {
        let mut b = VersionBumper::parse("1.2.3").unwrap();
        b.bump(BumpPart::Minor);
        assert_eq!(b.version_string(), "1.3.0");
    }

    #[test]
    fn test_bump_patch() {
        let mut b = VersionBumper::parse("1.2.3").unwrap();
        b.bump(BumpPart::Patch);
        assert_eq!(b.version_string(), "1.2.4");
    }

    #[test]
    fn test_bump_patch_strips_pre() {
        let mut b = VersionBumper::parse("1.0.0-alpha.1").unwrap();
        b.bump(BumpPart::Patch);
        assert_eq!(b.version_string(), "1.0.0");
    }

    #[test]
    fn test_bump_pre_from_release() {
        let mut b = VersionBumper::parse("1.0.0").unwrap();
        b.bump(BumpPart::PreRelease);
        assert_eq!(b.version_string(), "1.0.0-alpha.0");
    }

    #[test]
    fn test_bump_pre_from_pre() {
        let mut b = VersionBumper::parse("1.0.0-alpha.0").unwrap();
        b.bump(BumpPart::PreRelease);
        assert_eq!(b.version_string(), "1.0.0-alpha.1");
    }

    #[test]
    fn test_bump_major_resets_pre() {
        let mut b = VersionBumper::parse("1.0.0-beta.5").unwrap();
        b.bump(BumpPart::Major);
        assert_eq!(b.version_string(), "2.0.0");
    }

    #[test]
    fn test_bump_part_parse() {
        use std::str::FromStr;
        assert_eq!(BumpPart::from_str("major").unwrap(), BumpPart::Major);
        assert_eq!(BumpPart::from_str("minor").unwrap(), BumpPart::Minor);
        assert_eq!(BumpPart::from_str("patch").unwrap(), BumpPart::Patch);
        assert_eq!(BumpPart::from_str("pre").unwrap(), BumpPart::PreRelease);
        assert!(BumpPart::from_str("invalid").is_err());
    }
}
