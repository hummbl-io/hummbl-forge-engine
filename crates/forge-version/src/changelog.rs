//! Changelog generation from version history.
//!
//! Maintains a CHANGELOG.md file that records changes per version.

use std::path::Path;

/// A changelog entry for a single version.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChangelogEntry {
    pub version: String,
    pub date: String,
    #[serde(default)]
    pub added: Vec<String>,
    #[serde(default)]
    pub changed: Vec<String>,
    #[serde(default)]
    pub fixed: Vec<String>,
    #[serde(default)]
    pub removed: Vec<String>,
}

/// The full changelog.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Changelog {
    #[serde(default)]
    pub entries: Vec<ChangelogEntry>,
}

impl Changelog {
    /// Load a changelog from a JSON file.
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        if !path.exists() {
            return Ok(Self {
                entries: Vec::new(),
            });
        }
        let contents = std::fs::read_to_string(path)?;
        let changelog: Self = serde_json::from_str(&contents)
            .map_err(|e| anyhow::anyhow!("failed to parse changelog: {e}"))?;
        Ok(changelog)
    }

    /// Save the changelog to a JSON file.
    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Add a new version entry.
    pub fn add_entry(&mut self, entry: ChangelogEntry) {
        // Insert at the beginning (newest first)
        self.entries.insert(0, entry);
    }

    /// Render the changelog as Markdown.
    pub fn to_markdown(&self) -> String {
        let mut md = String::from("# Changelog\n\n");
        md.push_str("All notable changes to this project are documented here.\n\n");

        for entry in &self.entries {
            md.push_str(&format!("## [{}] - {}\n\n", entry.version, entry.date));

            if !entry.added.is_empty() {
                md.push_str("### Added\n");
                for item in &entry.added {
                    md.push_str(&format!("- {item}\n"));
                }
                md.push('\n');
            }
            if !entry.changed.is_empty() {
                md.push_str("### Changed\n");
                for item in &entry.changed {
                    md.push_str(&format!("- {item}\n"));
                }
                md.push('\n');
            }
            if !entry.fixed.is_empty() {
                md.push_str("### Fixed\n");
                for item in &entry.fixed {
                    md.push_str(&format!("- {item}\n"));
                }
                md.push('\n');
            }
            if !entry.removed.is_empty() {
                md.push_str("### Removed\n");
                for item in &entry.removed {
                    md.push_str(&format!("- {item}\n"));
                }
                md.push('\n');
            }
        }

        md
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_changelog_markdown() {
        let changelog = Changelog {
            entries: vec![ChangelogEntry {
                version: "0.1.0".to_string(),
                date: "2026-09-01".to_string(),
                added: vec!["Initial release".to_string()],
                changed: vec![],
                fixed: vec![],
                removed: vec![],
            }],
        };

        let md = changelog.to_markdown();
        assert!(md.contains("# Changelog"));
        assert!(md.contains("## [0.1.0] - 2026-09-01"));
        assert!(md.contains("### Added"));
        assert!(md.contains("- Initial release"));
    }

    #[test]
    fn test_changelog_save_load() {
        let changelog = Changelog {
            entries: vec![ChangelogEntry {
                version: "0.2.0".to_string(),
                date: "2026-09-02".to_string(),
                added: vec!["New feature".to_string()],
                changed: vec!["Updated API".to_string()],
                fixed: vec![],
                removed: vec![],
            }],
        };

        let tmp = std::env::temp_dir().join("forge_changelog_test.json");
        changelog.save(&tmp).unwrap();
        let loaded = Changelog::load(&tmp).unwrap();
        assert_eq!(loaded.entries.len(), 1);
        assert_eq!(loaded.entries[0].version, "0.2.0");
        assert_eq!(loaded.entries[0].added.len(), 1);
        assert_eq!(loaded.entries[0].changed.len(), 1);
        std::fs::remove_file(&tmp).ok();
    }
}
