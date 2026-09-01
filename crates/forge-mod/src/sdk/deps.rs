//! Dependency resolution — resolves mod load order from dependencies.
//!
//! Implements topological sort to determine the correct load order
//! for mods based on their declared dependencies.

use std::collections::{HashMap, HashSet};

use crate::manifest::ModManifest;

/// The dependency resolver determines the correct load order for mods.
pub struct DependencyResolver {
    mods: HashMap<String, ModManifest>,
}

impl DependencyResolver {
    /// Create a new resolver with a list of mod manifests.
    pub fn new(manifests: Vec<ModManifest>) -> Self {
        let mods = manifests
            .into_iter()
            .map(|m| (m.mod_info.id.clone(), m))
            .collect();
        Self { mods }
    }

    /// Resolve the load order.
    ///
    /// Returns an error if there's a circular dependency or a missing
    /// required dependency.
    pub fn resolve(&self) -> Result<ResolutionResult, DependencyError> {
        let mut order = Vec::new();
        let mut visited = HashSet::new();
        let mut visiting = HashSet::new();
        let mut missing = Vec::new();

        // Get all mod IDs
        let all_ids: Vec<String> = self.mods.keys().cloned().collect();

        for id in &all_ids {
            if !visited.contains(id) {
                self.visit(id, &mut order, &mut visited, &mut visiting, &mut missing)?;
            }
        }

        if !missing.is_empty() {
            return Err(DependencyError::MissingDependencies(missing));
        }

        Ok(ResolutionResult {
            load_order: order,
            skipped: Vec::new(),
        })
    }

    /// Visit a mod for topological sort.
    fn visit(
        &self,
        id: &str,
        order: &mut Vec<String>,
        visited: &mut HashSet<String>,
        visiting: &mut HashSet<String>,
        missing: &mut Vec<MissingDependency>,
    ) -> Result<(), DependencyError> {
        if visited.contains(id) {
            return Ok(());
        }

        if visiting.contains(id) {
            return Err(DependencyError::CircularDependency {
                cycle: format!("detected at mod: {id}"),
            });
        }

        visiting.insert(id.to_string());

        // Visit dependencies first
        if let Some(manifest) = self.mods.get(id) {
            for dep in &manifest.dependencies {
                if dep.optional {
                    // Skip optional deps — they don't need to be present
                    if self.mods.contains_key(&dep.id) {
                        self.visit(&dep.id, order, visited, visiting, missing)?;
                    }
                    continue;
                }

                if !self.mods.contains_key(&dep.id) {
                    missing.push(MissingDependency {
                        mod_id: id.to_string(),
                        missing_dep_id: dep.id.clone(),
                        required_version: dep.version.clone(),
                    });
                    continue;
                }

                self.visit(&dep.id, order, visited, visiting, missing)?;
            }
        }

        visiting.remove(id);
        visited.insert(id.to_string());
        order.push(id.to_string());

        Ok(())
    }
}

/// Result of dependency resolution.
#[derive(Debug)]
pub struct ResolutionResult {
    /// Mods in load order (dependencies first)
    pub load_order: Vec<String>,
    /// Mods that were skipped (optional deps not present)
    pub skipped: Vec<String>,
}

/// A missing dependency.
#[derive(Debug, Clone)]
pub struct MissingDependency {
    pub mod_id: String,
    pub missing_dep_id: String,
    pub required_version: String,
}

/// Dependency resolution errors.
#[derive(Debug, thiserror::Error)]
pub enum DependencyError {
    #[error("circular dependency: {cycle}")]
    CircularDependency { cycle: String },
    #[error("missing dependencies: {0:?}")]
    MissingDependencies(Vec<MissingDependency>),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{ModDependency, ModInfo, ModTarget};

    fn make_manifest(id: &str, deps: Vec<ModDependency>) -> ModManifest {
        ModManifest {
            mod_info: ModInfo {
                id: id.to_string(),
                name: id.to_string(),
                author: "test".to_string(),
                version: "1.0.0".to_string(),
                description: String::new(),
            },
            target: ModTarget {
                game: "test".to_string(),
                compatible_versions: ">=1.0.0".to_string(),
            },
            dependencies: deps,
            hooks: vec![],
            permissions: vec![],
        }
    }

    fn make_dep(id: &str, optional: bool) -> ModDependency {
        ModDependency {
            id: id.to_string(),
            version: ">=1.0.0".to_string(),
            optional,
        }
    }

    #[test]
    fn test_no_dependencies() {
        let resolver =
            DependencyResolver::new(vec![make_manifest("a", vec![]), make_manifest("b", vec![])]);
        let result = resolver.resolve().unwrap();
        assert_eq!(result.load_order.len(), 2);
    }

    #[test]
    fn test_simple_dependency_order() {
        // b depends on a, so a should load first
        let resolver = DependencyResolver::new(vec![
            make_manifest("a", vec![]),
            make_manifest("b", vec![make_dep("a", false)]),
        ]);
        let result = resolver.resolve().unwrap();
        assert_eq!(result.load_order[0], "a");
        assert_eq!(result.load_order[1], "b");
    }

    #[test]
    fn test_missing_required_dependency() {
        let resolver =
            DependencyResolver::new(vec![make_manifest("b", vec![make_dep("a", false)])]);
        let result = resolver.resolve();
        assert!(result.is_err());
    }

    #[test]
    fn test_optional_dependency_missing() {
        // b optionally depends on a, a is missing — should still resolve
        let resolver = DependencyResolver::new(vec![make_manifest("b", vec![make_dep("a", true)])]);
        let result = resolver.resolve().unwrap();
        assert_eq!(result.load_order, vec!["b"]);
    }

    #[test]
    fn test_circular_dependency() {
        // a depends on b, b depends on a
        let resolver = DependencyResolver::new(vec![
            make_manifest("a", vec![make_dep("b", false)]),
            make_manifest("b", vec![make_dep("a", false)]),
        ]);
        let result = resolver.resolve();
        assert!(result.is_err());
    }

    #[test]
    fn test_chain_dependency() {
        // c depends on b, b depends on a
        let resolver = DependencyResolver::new(vec![
            make_manifest("a", vec![]),
            make_manifest("b", vec![make_dep("a", false)]),
            make_manifest("c", vec![make_dep("b", false)]),
        ]);
        let result = resolver.resolve().unwrap();
        assert_eq!(result.load_order, vec!["a", "b", "c"]);
    }
}
