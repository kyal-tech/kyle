use kyc_core::resolver::ResolvedGraph;
use kyc_core::semver::version_to_string;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockEntry {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub checksum: String,
    #[serde(default)]
    pub source: String,
    /// Dependencies of this package (name only, version in their own entry).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockFile {
    pub version: i32,
    #[serde(default)]
    pub packages: Vec<LockEntry>,
}

impl LockFile {
    pub fn read(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read lock file: {}", e))?;
        toml::from_str(&content)
            .map_err(|e| format!("Failed to parse lock file: {}", e))
    }

    pub fn write(&self, path: &Path) -> Result<(), String> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize lock file: {}", e))?;
        fs::write(path, &content)
            .map_err(|e| format!("Failed to write lock file: {}", e))
    }

    pub fn add_package(&mut self, name: &str, version: &str, source: &str) {
        self.packages.retain(|p| p.name != name);
        self.packages.push(LockEntry {
            name: name.to_string(),
            version: version.to_string(),
            checksum: String::new(),
            source: source.to_string(),
            dependencies: Vec::new(),
        });
    }

    pub fn add_package_with_deps(
        &mut self,
        name: &str,
        version: &str,
        source: &str,
        deps: &[String],
    ) {
        self.packages.retain(|p| p.name != name);
        self.packages.push(LockEntry {
            name: name.to_string(),
            version: version.to_string(),
            checksum: String::new(),
            source: source.to_string(),
            dependencies: deps.to_vec(),
        });
    }

    pub fn remove_package(&mut self, name: &str) -> bool {
        let before = self.packages.len();
        self.packages.retain(|p| p.name != name);
        self.packages.len() != before
    }

    /// Get the resolved version of a package.
    pub fn get_version(&self, name: &str) -> Option<&str> {
        self.packages.iter()
            .find(|p| p.name == name)
            .map(|p| p.version.as_str())
    }

    /// Check if lock file matches the manifest dependencies.
    /// Returns true if every dependency in manifest has a resolved entry.
    pub fn matches_manifest(&self, manifest_deps: &HashMap<String, String>) -> bool {
        manifest_deps.keys().all(|name| self.packages.iter().any(|p| p.name == *name))
    }

    /// Get all package names in dependency order (topological).
    pub fn dependency_order(&self) -> Vec<String> {
        let names: HashSet<String> = self.packages.iter().map(|p| p.name.clone()).collect();
        let dep_map: HashMap<&str, &[String]> = self.packages.iter()
            .map(|p| (p.name.as_str(), p.dependencies.as_slice()))
            .collect();

        let mut sorted: Vec<String> = Vec::new();
        let mut visited: HashSet<&str> = HashSet::new();
        let mut visiting: HashSet<&str> = HashSet::new();

        fn visit<'a>(
            name: &'a str,
            dep_map: &'a HashMap<&'a str, &'a [String]>,
            sorted: &mut Vec<String>,
            visited: &mut HashSet<&'a str>,
            visiting: &mut HashSet<&'a str>,
            all_names: &'a HashSet<String>,
        ) {
            if visited.contains(name) {
                return;
            }
            if visiting.contains(name) {
                return; // cycle
            }
            visiting.insert(name);
            if let Some(deps) = dep_map.get(name) {
                for dep_name in *deps {
                    if all_names.contains(dep_name) {
                        visit(dep_name, dep_map, sorted, visited, visiting, all_names);
                    }
                }
            }
            visiting.remove(name);
            visited.insert(name);
            sorted.push(name.to_string());
        }

        let mut all_sorted: Vec<String> = names.iter().cloned().collect();
        all_sorted.sort();
        for name in &all_sorted {
            visit(name.as_str(), &dep_map, &mut sorted, &mut visited, &mut visiting, &names);
        }

        sorted
    }

    /// Update the lock file from a resolved dependency graph.
    pub fn update_from_graph(&mut self, graph: &ResolvedGraph, source: &str) {
        let resolved: HashMap<&str, Vec<String>> = graph.packages.values()
            .map(|p| {
                let deps: Vec<String> = p.dependencies.iter()
                    .filter(|d| graph.packages.contains_key(&d.name))
                    .map(|d| d.name.clone())
                    .collect();
                (p.name.as_str(), deps)
            })
            .collect();

        // Add entries in topological order
        let mut new_packages: Vec<LockEntry> = Vec::new();
        let mut seen: HashSet<&str> = HashSet::new();

        for name in &graph.order {
            if let Some(pkg) = graph.packages.get(name) {
                let deps = resolved.get(name.as_str()).cloned().unwrap_or_default();
                new_packages.push(LockEntry {
                    name: name.clone(),
                    version: version_to_string(&pkg.version),
                    checksum: String::new(),
                    source: source.to_string(),
                    dependencies: deps,
                });
                seen.insert(name.as_str());
            }
        }

        // Add any packages not in topological order (shouldn't happen, but be safe)
        for pkg in graph.packages.values() {
            if !seen.contains(pkg.name.as_str()) {
                let deps = pkg.dependencies.iter()
                    .filter(|d| graph.packages.contains_key(&d.name))
                    .map(|d| d.name.clone())
                    .collect();
                new_packages.push(LockEntry {
                    name: pkg.name.clone(),
                    version: version_to_string(&pkg.version),
                    checksum: String::new(),
                    source: source.to_string(),
                    dependencies: deps,
                });
            }
        }

        self.packages = new_packages;
    }
}

impl Default for LockFile {
    fn default() -> Self {
        Self {
            version: 1,
            packages: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lock_default() {
        let lock = LockFile::default();
        assert_eq!(lock.version, 1);
        assert!(lock.packages.is_empty());
    }

    #[test]
    fn test_add_and_get_package() {
        let mut lock = LockFile::default();
        lock.add_package("foo", "1.0.0", "registry");
        assert_eq!(lock.get_version("foo").unwrap(), "1.0.0");
    }

    #[test]
    fn test_add_package_with_deps() {
        let mut lock = LockFile::default();
        lock.add_package_with_deps("foo", "1.0.0", "registry", &["bar".to_string()]);
        let entry = lock.packages.iter().find(|p| p.name == "foo").unwrap();
        assert_eq!(entry.dependencies, vec!["bar"]);
    }

    #[test]
    fn test_remove_package() {
        let mut lock = LockFile::default();
        lock.add_package("foo", "1.0.0", "registry");
        assert!(lock.remove_package("foo"));
        assert!(!lock.remove_package("nonexistent"));
    }

    #[test]
    fn test_matches_manifest() {
        let mut lock = LockFile::default();
        lock.add_package("foo", "1.0.0", "registry");

        let mut deps = HashMap::new();
        deps.insert("foo".to_string(), "1.0.0".to_string());
        assert!(lock.matches_manifest(&deps));

        deps.insert("bar".to_string(), "2.0.0".to_string());
        assert!(!lock.matches_manifest(&deps));
    }

    #[test]
    fn test_dependency_order() {
        let mut lock = LockFile::default();
        lock.add_package_with_deps("app", "1.0.0", "registry", &["lib_b".to_string(), "lib_a".to_string()]);
        lock.add_package_with_deps("lib_a", "1.0.0", "registry", &["base".to_string()]);
        lock.add_package_with_deps("lib_b", "1.0.0", "registry", &[]);
        lock.add_package_with_deps("base", "1.0.0", "registry", &[]);

        let order = lock.dependency_order();
        // base must come before lib_a, which must come before app
        let base_pos = order.iter().position(|n| n == "base").unwrap();
        let lib_a_pos = order.iter().position(|n| n == "lib_a").unwrap();
        let lib_b_pos = order.iter().position(|n| n == "lib_b").unwrap();
        let app_pos = order.iter().position(|n| n == "app").unwrap();

        assert!(base_pos < lib_a_pos);
        assert!(lib_b_pos < app_pos);
        assert!(lib_a_pos < app_pos);
    }
}
