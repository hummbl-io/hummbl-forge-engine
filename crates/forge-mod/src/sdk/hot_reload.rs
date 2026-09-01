//! Hot-reload support — watches mod files and reloads them on change.
//!
//! Uses file system watching to detect changes to mod files and
//! triggers a reload of the affected mod.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;

/// Type alias for reload callback functions.
type ReloadCallback = Box<dyn Fn(&str) + Send + Sync>;

/// Watches mod directories for changes and triggers reloads.
pub struct HotReloader {
    /// Map of watched paths to their last modification time
    watched: HashMap<PathBuf, SystemTime>,
    /// Callbacks to invoke when a mod is reloaded
    callbacks: Vec<ReloadCallback>,
}

impl HotReloader {
    /// Create a new hot-reloader.
    pub fn new() -> Self {
        Self {
            watched: HashMap::new(),
            callbacks: Vec::new(),
        }
    }

    /// Register a mod directory to watch.
    pub fn watch(&mut self, mod_id: &str, path: PathBuf) {
        if let Ok(metadata) = std::fs::metadata(&path) {
            if let Ok(modified) = metadata.modified() {
                self.watched.insert(path, modified);
                tracing::debug!("Watching mod {mod_id}");
            }
        }
    }

    /// Register a callback to invoke when a mod is reloaded.
    pub fn on_reload(&mut self, callback: impl Fn(&str) + Send + Sync + 'static) {
        self.callbacks.push(Box::new(callback));
    }

    /// Check all watched paths for changes.
    /// Returns a list of mod IDs that were reloaded.
    pub fn check(&mut self) -> Vec<String> {
        let mut reloaded = Vec::new();
        let mut updates = Vec::new();

        for (path, last_modified) in &self.watched {
            if let Ok(metadata) = std::fs::metadata(path) {
                if let Ok(modified) = metadata.modified() {
                    if modified != *last_modified {
                        // File changed — extract mod ID from path
                        let mod_id = path
                            .parent()
                            .and_then(|p| p.file_name())
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown");
                        reloaded.push(mod_id.to_string());
                        updates.push((path.clone(), modified));
                    }
                }
            }
        }

        // Update modification times
        for (path, modified) in updates {
            self.watched.insert(path, modified);
        }

        // Invoke callbacks
        for mod_id in &reloaded {
            tracing::info!("Hot-reloading mod: {mod_id}");
            for callback in &self.callbacks {
                callback(mod_id);
            }
        }

        reloaded
    }

    /// Stop watching a mod.
    pub fn unwatch(&mut self, path: &PathBuf) {
        self.watched.remove(path);
    }

    /// Get the number of watched mods.
    pub fn watched_count(&self) -> usize {
        self.watched.len()
    }
}

impl Default for HotReloader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watch_and_check_no_change() {
        let mut reloader = HotReloader::new();
        let tmp = std::env::temp_dir().join("forge_hotreload_test.txt");
        std::fs::write(&tmp, "test").unwrap();

        reloader.watch("test-mod", tmp.clone());
        assert_eq!(reloader.watched_count(), 1);

        let changed = reloader.check();
        assert!(changed.is_empty()); // no changes yet

        std::fs::remove_file(&tmp).ok();
    }

    #[test]
    fn test_watch_and_detect_change() {
        let mut reloader = HotReloader::new();
        let tmp = std::env::temp_dir().join("forge_hotreload_test2.txt");
        std::fs::write(&tmp, "initial").unwrap();

        reloader.watch("test-mod", tmp.clone());

        // No change yet
        assert!(reloader.check().is_empty());

        // Wait a moment and modify
        std::thread::sleep(std::time::Duration::from_millis(10));
        std::fs::write(&tmp, "modified").unwrap();

        let changed = reloader.check();
        assert!(!changed.is_empty());

        std::fs::remove_file(&tmp).ok();
    }
}
