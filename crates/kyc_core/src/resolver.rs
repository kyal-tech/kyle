use crate::semver::{
    parse_requirement, version_to_string, SemanticVersion, VersionRequirement,
};
use std::collections::{HashMap, HashSet};

/// A single resolved package entry.
#[derive(Debug, Clone)]
pub struct ResolvedPackage {
    pub name: String,
    pub version: SemanticVersion,
    pub dependencies: Vec<DepSpec>,
}

/// A dependency specification (name + version requirement).
#[derive(Debug, Clone)]
pub struct DepSpec {
    pub name: String,
    pub requirement: VersionRequirement,
}

/// The full resolved dependency graph.
#[derive(Debug, Clone)]
pub struct ResolvedGraph {
    /// All resolved packages, keyed by name.
    pub packages: HashMap<String, ResolvedPackage>,
    /// Resolution order (topological): packages that depend on nothing first.
    pub order: Vec<String>,
    /// Any conflicts found during resolution.
    pub conflicts: Vec<String>,
}

/// Registry interface — abstracts how we query available package versions.
/// This is implemented by the actual registry client in kyc_tools.
pub trait RegistryBackend {
    fn get_versions(&self, name: &str) -> Result<Vec<SemanticVersion>, String>;
    fn get_dependencies(&self, name: &str, version: &SemanticVersion) -> Result<Vec<DepSpec>, String>;
}

impl ResolvedGraph {
    pub fn new() -> Self {
        Self {
            packages: HashMap::new(),
            order: Vec::new(),
            conflicts: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.packages.is_empty()
    }

    pub fn resolved_version(&self, name: &str) -> Option<&SemanticVersion> {
        self.packages.get(name).map(|p| &p.version)
    }

    pub fn version_str(&self, name: &str) -> Option<String> {
        self.resolved_version(name).map(version_to_string)
    }
}

/// Greedy dependency resolver.
///
/// Algorithm:
/// 1. For each root dependency, query the registry for all versions.
/// 2. Filter by version requirement, pick the highest matching.
/// 3. Recursively resolve transitive dependencies.
/// 4. If a package appears twice with incompatible requirements → conflict.
/// 5. Produce a topological order for compilation.
pub fn resolve(
    roots: &[DepSpec],
    registry: &dyn RegistryBackend,
    max_depth: u32,
) -> Result<ResolvedGraph, String> {
    let mut graph = ResolvedGraph::new();
    let mut visiting: HashSet<String> = HashSet::new();
    let mut visited: HashSet<String> = HashSet::new();

    for root in roots {
        resolve_dep(root, registry, &mut graph, &mut visiting, &mut visited, 0, max_depth)?;
    }

    // Topological sort
    graph.order = topological_sort(&graph);

    Ok(graph)
}

fn resolve_dep(
    dep: &DepSpec,
    registry: &dyn RegistryBackend,
    graph: &mut ResolvedGraph,
    visiting: &mut HashSet<String>,
    visited: &mut HashSet<String>,
    depth: u32,
    max_depth: u32,
) -> Result<(), String> {
    if depth > max_depth {
        return Err(format!(
            "Maximum dependency depth ({}) exceeded while resolving '{}'",
            max_depth, dep.name
        ));
    }

    let name = &dep.name;

    // Check if already resolved
    if let Some(existing) = graph.packages.get(name) {
        let existing_ver = &existing.version;
        if !dep.requirement.matches(existing_ver) {
            graph.conflicts.push(format!(
                "conflict: '{}' requires {} but '{}' is already resolved as {}",
                name, dep.requirement, name, version_to_string(existing_ver)
            ));
            return Err(format!(
                "Version conflict for '{}': requires {} but {} already resolved",
                name, dep.requirement, version_to_string(existing_ver)
            ));
        }
        return Ok(());
    }

    // Cycle detection
    if visiting.contains(name) {
        return Err(format!("Circular dependency detected: '{}'", name));
    }
    if visited.contains(name) {
        return Ok(());
    }

    visiting.insert(name.clone());

    // Query registry
    let versions = registry.get_versions(name)?;
    if versions.is_empty() {
        return Err(format!("Package '{}' not found in registry", name));
    }

    // Filter and pick highest compatible version
    let compatible: Vec<&SemanticVersion> = versions.iter()
        .filter(|v| dep.requirement.matches(v))
        .collect();

    let chosen = compatible.into_iter().max().ok_or_else(|| {
        format!(
            "No compatible version found for '{}' matching {} (available: {})",
            name,
            dep.requirement,
            versions.iter()
                .map(version_to_string)
                .collect::<Vec<_>>()
                .join(", ")
        )
    })?;

    let chosen_version = chosen.clone();

    // Get transitive deps
    let transitive = registry.get_dependencies(name, &chosen_version)?;

    let resolved = ResolvedPackage {
        name: name.clone(),
        version: chosen_version,
        dependencies: transitive.clone(),
    };

    graph.packages.insert(name.clone(), resolved);
    visiting.remove(name);
    visited.insert(name.clone());

    // Recurse into transitive dependencies
    for sub_dep in &transitive {
        resolve_dep(sub_dep, registry, graph, visiting, visited, depth + 1, max_depth)?;
    }

    Ok(())
}

/// Topological sort: packages with no deps first, then their dependents.
fn topological_sort(graph: &ResolvedGraph) -> Vec<String> {
    let mut sorted: Vec<String> = Vec::new();
    let mut permanent_mark: HashSet<String> = HashSet::new();
    let mut temporary_mark: HashSet<String> = HashSet::new();

    fn visit(
        name: &str,
        graph: &ResolvedGraph,
        sorted: &mut Vec<String>,
        permanent: &mut HashSet<String>,
        temporary: &mut HashSet<String>,
    ) {
        if permanent.contains(name) {
            return;
        }
        if temporary.contains(name) {
            return; // cycle, skip
        }
        temporary.insert(name.to_string());

        if let Some(pkg) = graph.packages.get(name) {
            for dep in &pkg.dependencies {
                if graph.packages.contains_key(&dep.name) {
                    visit(&dep.name, graph, sorted, permanent, temporary);
                }
            }
        }

        temporary.remove(name);
        permanent.insert(name.to_string());
        sorted.push(name.to_string());
    }

    let mut names: Vec<String> = graph.packages.keys().cloned().collect();
    names.sort();
    for name in &names {
        visit(name, graph, &mut sorted, &mut permanent_mark, &mut temporary_mark);
    }

    sorted
}

/// Resolve from a set of dependency specs (name → requirement string).
pub fn resolve_from_strings(
    deps: &HashMap<String, String>,
    registry: &dyn RegistryBackend,
) -> Result<ResolvedGraph, String> {
    let mut roots: Vec<DepSpec> = Vec::new();
    for (name, req_str) in deps {
        let req = parse_requirement(req_str)
            .map_err(|e| format!("Invalid requirement for '{}': {}", name, e))?;
        roots.push(DepSpec {
            name: name.clone(),
            requirement: req,
        });
    }
    resolve(&roots, registry, 100)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semver::parse_version;

    struct MockRegistry {
        packages: HashMap<String, Vec<(SemanticVersion, Vec<DepSpec>)>>,
    }

    impl MockRegistry {
        fn new() -> Self {
            Self { packages: HashMap::new() }
        }

        fn add(&mut self, name: &str, version: &str, deps: &[(&str, &str)]) {
            let ver = parse_version(version).unwrap();
            let dep_specs: Vec<DepSpec> = deps.iter().map(|(n, r)| {
                DepSpec {
                    name: n.to_string(),
                    requirement: parse_requirement(r).unwrap(),
                }
            }).collect();
            self.packages.entry(name.to_string())
                .or_default()
                .push((ver, dep_specs));
        }
    }

    impl RegistryBackend for MockRegistry {
        fn get_versions(&self, name: &str) -> Result<Vec<SemanticVersion>, String> {
            Ok(self.packages.get(name)
                .map(|v| v.iter().map(|(ver, _)| ver.clone()).collect())
                .unwrap_or_default())
        }

        fn get_dependencies(&self, name: &str, version: &SemanticVersion) -> Result<Vec<DepSpec>, String> {
            Ok(self.packages.get(name)
                .and_then(|v| v.iter().find(|(ver, _)| ver == version))
                .map(|(_, deps)| deps.clone())
                .unwrap_or_default())
        }
    }

    #[test]
    fn test_resolve_simple() {
        let mut reg = MockRegistry::new();
        reg.add("foo", "1.0.0", &[]);
        reg.add("foo", "2.0.0", &[]);

        let mut deps = HashMap::new();
        deps.insert("foo".to_string(), "^1.0.0".to_string());

        let graph = resolve_from_strings(&deps, &reg).unwrap();
        assert_eq!(graph.version_str("foo").unwrap(), "1.0.0");
        assert!(graph.conflicts.is_empty());
    }

    #[test]
    fn test_resolve_highest_compatible() {
        let mut reg = MockRegistry::new();
        reg.add("foo", "1.0.0", &[]);
        reg.add("foo", "1.5.0", &[]);
        reg.add("foo", "1.9.0", &[]);
        reg.add("foo", "2.0.0", &[]);

        let mut deps = HashMap::new();
        deps.insert("foo".to_string(), "^1.0.0".to_string());

        let graph = resolve_from_strings(&deps, &reg).unwrap();
        assert_eq!(graph.version_str("foo").unwrap(), "1.9.0");
    }

    #[test]
    fn test_resolve_exact_version() {
        let mut reg = MockRegistry::new();
        reg.add("foo", "1.0.0", &[]);
        reg.add("foo", "1.5.0", &[]);

        let mut deps = HashMap::new();
        deps.insert("foo".to_string(), "=1.0.0".to_string());

        let graph = resolve_from_strings(&deps, &reg).unwrap();
        assert_eq!(graph.version_str("foo").unwrap(), "1.0.0");
    }

    #[test]
    fn test_resolve_transitive_deps() {
        let mut reg = MockRegistry::new();
        reg.add("foo", "1.0.0", &[("bar", ">=1.0")]);
        reg.add("bar", "1.0.0", &[]);
        reg.add("bar", "2.0.0", &[]);

        let mut deps = HashMap::new();
        deps.insert("foo".to_string(), "1.0.0".to_string());

        let graph = resolve_from_strings(&deps, &reg).unwrap();
        assert_eq!(graph.version_str("foo").unwrap(), "1.0.0");
        assert_eq!(graph.version_str("bar").unwrap(), "2.0.0");
        assert!(graph.order.contains(&"bar".to_string()));
        assert!(graph.order.contains(&"foo".to_string()));
        // bar should come before foo in topological order
        let bar_idx = graph.order.iter().position(|n| n == "bar").unwrap();
        let foo_idx = graph.order.iter().position(|n| n == "foo").unwrap();
        assert!(bar_idx < foo_idx, "bar must be resolved before foo");
    }

    #[test]
    fn test_resolve_version_conflict() {
        let mut reg = MockRegistry::new();
        reg.add("shared", "1.0.0", &[]);
        reg.add("shared", "2.0.0", &[]);
        reg.add("foo", "1.0.0", &[("shared", "^1.0")]);
        reg.add("bar", "1.0.0", &[("shared", "^2.0")]);

        let mut deps = HashMap::new();
        deps.insert("foo".to_string(), "1.0.0".to_string());
        deps.insert("bar".to_string(), "1.0.0".to_string());

        let err = resolve_from_strings(&deps, &reg).unwrap_err();
        assert!(err.contains("conflict") || err.contains("Conflict"));
    }

    #[test]
    fn test_resolve_package_not_found() {
        let reg = MockRegistry::new();
        let mut deps = HashMap::new();
        deps.insert("nonexistent".to_string(), "1.0.0".to_string());

        let result = resolve_from_strings(&deps, &reg);
        assert!(result.is_err());
    }

    #[test]
    fn test_topological_order_single() {
        let mut reg = MockRegistry::new();
        reg.add("foo", "1.0.0", &[]);

        let mut deps = HashMap::new();
        deps.insert("foo".to_string(), "1.0.0".to_string());

        let graph = resolve_from_strings(&deps, &reg).unwrap();
        assert_eq!(graph.order, vec!["foo"]);
    }

    #[test]
    fn test_topological_order_chain() {
        let mut reg = MockRegistry::new();
        reg.add("a", "1.0.0", &[("b", "1.0")]);
        reg.add("b", "1.0.0", &[("c", "1.0")]);
        reg.add("c", "1.0.0", &[]);

        let mut deps = HashMap::new();
        deps.insert("a".to_string(), "1.0.0".to_string());

        let graph = resolve_from_strings(&deps, &reg).unwrap();
        let order: Vec<&str> = graph.order.iter().map(|s| s.as_str()).collect();
        // c must come before b, b before a
        let c_pos = order.iter().position(|&n| n == "c").unwrap();
        let b_pos = order.iter().position(|&n| n == "b").unwrap();
        let a_pos = order.iter().position(|&n| n == "a").unwrap();
        assert!(c_pos < b_pos, "c must come before b");
        assert!(b_pos < a_pos, "b must come before a");
    }

    #[test]
    fn test_empty_deps() {
        let reg = MockRegistry::new();
        let deps = HashMap::new();

        let graph = resolve_from_strings(&deps, &reg).unwrap();
        assert!(graph.is_empty());
        assert!(graph.order.is_empty());
    }

    #[test]
    fn test_wildcard_requirement() {
        let mut reg = MockRegistry::new();
        reg.add("foo", "1.0.0", &[]);
        reg.add("foo", "2.0.0", &[]);
        reg.add("foo", "3.0.0", &[]);

        let mut deps = HashMap::new();
        deps.insert("foo".to_string(), "*".to_string());

        let graph = resolve_from_strings(&deps, &reg).unwrap();
        assert_eq!(graph.version_str("foo").unwrap(), "3.0.0");
    }
}
