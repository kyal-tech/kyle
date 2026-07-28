use kyc_core::resolver::{resolve_from_strings, RegistryBackend, ResolvedGraph};
use kyc_core::semver::{parse_version, SemanticVersion};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Project configuration — optional `[project]` table.
/// Fields here override top-level flat fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub edition: String,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub license: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub main: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    // Flat fields (backward compatible)
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub edition: String,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub license: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub main: String,

    // Optional [project] table (new format, overrides flat fields)
    #[serde(default)]
    pub project: Option<ProjectConfig>,

    #[serde(default)]
    pub compiler: CompilerConfig,

    #[serde(default)]
    pub dependencies: HashMap<String, String>,

    #[serde(default, rename = "dev-dependencies")]
    pub dev_dependencies: HashMap<String, String>,

    #[serde(default)]
    pub format: FormatConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilerConfig {
    #[serde(default = "default_optimization")]
    pub optimization: String,
    #[serde(default = "default_target")]
    pub target: String,
    #[serde(default)]
    pub debug: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatConfig {
    #[serde(default = "default_max_line_width")]
    pub max_line_width: usize,
    #[serde(default = "default_indent_size")]
    pub indent_size: usize,
    #[serde(default = "default_trailing_newline")]
    pub trailing_newline: bool,
}

fn default_max_line_width() -> usize { 100 }
fn default_indent_size() -> usize { 4 }
fn default_trailing_newline() -> bool { true }

impl Default for FormatConfig {
    fn default() -> Self {
        Self {
            max_line_width: default_max_line_width(),
            indent_size: default_indent_size(),
            trailing_newline: default_trailing_newline(),
        }
    }
}

fn default_optimization() -> String { "O2".to_string() }
fn default_target() -> String { "native".to_string() }

impl Default for CompilerConfig {
    fn default() -> Self {
        Self {
            optimization: default_optimization(),
            target: default_target(),
            debug: false,
        }
    }
}

impl Manifest {
    // ── Resolved accessors (project table overrides flat fields) ──

    pub fn project_name(&self) -> &str {
        self.project.as_ref()
            .and_then(|p| if p.name.is_empty() { None } else { Some(p.name.as_str()) })
            .unwrap_or(&self.name)
    }

    pub fn project_version(&self) -> &str {
        self.project.as_ref()
            .and_then(|p| if p.version.is_empty() { None } else { Some(p.version.as_str()) })
            .unwrap_or(&self.version)
    }

    pub fn project_edition(&self) -> &str {
        self.project.as_ref()
            .and_then(|p| if p.edition.is_empty() { None } else { Some(p.edition.as_str()) })
            .unwrap_or(&self.edition)
    }

    pub fn project_authors(&self) -> &[String] {
        self.project.as_ref()
            .and_then(|p| if p.authors.is_empty() { None } else { Some(p.authors.as_slice()) })
            .unwrap_or(&self.authors)
    }

    pub fn project_license(&self) -> &str {
        self.project.as_ref()
            .and_then(|p| if p.license.is_empty() { None } else { Some(p.license.as_str()) })
            .unwrap_or(&self.license)
    }

    pub fn project_description(&self) -> &str {
        self.project.as_ref()
            .and_then(|p| if p.description.is_empty() { None } else { Some(p.description.as_str()) })
            .unwrap_or(&self.description)
    }

    pub fn project_main(&self) -> &str {
        self.project.as_ref()
            .and_then(|p| if p.main.is_empty() { None } else { Some(p.main.as_str()) })
            .unwrap_or(&self.main)
    }

    // ── I/O ──

    /// Parse manifest from TOML content.
    /// Preserves the original `toml::de::Error` for line/column info.
    pub fn from_str(content: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(content)
    }

    pub fn read(path: &Path) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
        Self::from_str(&content)
            .map_err(|e| format!("Failed to parse {}: {}\nTip: check for syntax errors in ky.toml", path.display(), e))
    }

    pub fn write(&self, path: &Path) -> Result<(), String> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize manifest: {}", e))?;
        fs::write(path, &content)
            .map_err(|e| format!("Failed to write {}: {}", path.display(), e))
    }

    pub fn find_in_directory(dir: &Path) -> Result<Self, String> {
        let manifest_path = dir.join("ky.toml");
        if !manifest_path.exists() {
            return Err(format!(
                "No ky.toml found in '{}'.\n\
                 Tip: run 'kl new <project>' to create a new project",
                dir.display()
            ));
        }
        Self::read(&manifest_path)
    }

    pub fn find_in_cwd() -> Result<Self, String> {
        Self::find_in_directory(&std::env::current_dir().map_err(|e| format!("{}", e))?)
    }

    pub fn add_dependency(&mut self, name: &str, version: &str) {
        self.dependencies.insert(name.to_string(), version.to_string());
    }

    pub fn remove_dependency(&mut self, name: &str) -> bool {
        self.dependencies.remove(name).is_some()
    }

    pub fn save_to_dir(&self, dir: &Path) -> Result<(), String> {
        self.write(&dir.join("ky.toml"))
    }

    // ── Validation ──

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors: Vec<String> = Vec::new();

        let name = self.project_name();
        let version = self.project_version();
        let edition = self.project_edition();

        if name.is_empty() {
            errors.push("project name is required (set 'name' or [project].name in ky.toml)".into());
        } else if !name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            errors.push(format!(
                "invalid project name '{}': only letters, numbers, _, - allowed.\n\
                 Tip: use 'my_project' or 'my-project' instead",
                name
            ));
        }

        if version.is_empty() {
            errors.push("project version is required (set 'version' or [project].version in ky.toml)".into());
        } else if let Err(e) = parse_version(version) {
            errors.push(format!(
                "invalid project version '{}': {}\n\
                 Tip: use semantic versioning like '0.1.0' or '1.2.3'",
                version, e
            ));
        }

        if !edition.is_empty() && edition != "2024" {
            errors.push(format!(
                "unsupported edition '{}' (supported: 2024).\n\
                 Tip: set edition = \"2024\" in ky.toml",
                edition
            ));
        }

        for (dep_name, dep_ver) in &self.dependencies {
            if dep_name.is_empty() {
                errors.push("dependency name cannot be empty".into());
            }
            if let Err(e) = kyc_core::semver::parse_requirement(dep_ver) {
                errors.push(format!(
                    "invalid version requirement for '{}': {} ({})\n\
                     Tip: use '1.0.0', '^1.0.0', '>=1.0 <2.0', or '*'",
                    dep_name, e, dep_ver
                ));
            }
        }

        for (dep_name, dep_ver) in &self.dev_dependencies {
            if dep_name.is_empty() {
                errors.push("dev-dependency name cannot be empty".into());
            }
            if let Err(e) = kyc_core::semver::parse_requirement(dep_ver) {
                errors.push(format!(
                    "invalid version requirement for dev-dep '{}': {} ({})",
                    dep_name, e, dep_ver
                ));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn parsed_version(&self) -> Result<SemanticVersion, String> {
        parse_version(self.project_version())
    }

    // ── Dependency Resolution ──

    /// Resolve all dependencies (including transitive) using the given registry backend.
    /// Returns a ResolvedGraph with all packages, versions, and topological order.
    pub fn resolve_dependencies(
        &self,
        registry: &dyn RegistryBackend,
    ) -> Result<ResolvedGraph, String> {
        resolve_from_strings(&self.dependencies, registry)
    }

    /// Resolve all dev-dependencies (includes regular deps too).
    pub fn resolve_all_dependencies(
        &self,
        registry: &dyn RegistryBackend,
    ) -> Result<ResolvedGraph, String> {
        let mut all_deps = self.dependencies.clone();
        for (name, ver) in &self.dev_dependencies {
            all_deps.entry(name.clone()).or_insert_with(|| ver.clone());
        }
        resolve_from_strings(&all_deps, registry)
    }

    /// Get all dependency specs as a string for display.
    pub fn dependency_summary(&self) -> Vec<String> {
        let mut lines: Vec<String> = self.dependencies.iter()
            .map(|(n, v)| format!("  {} = \"{}\"", n, v))
            .collect();
        if !self.dev_dependencies.is_empty() {
            if !lines.is_empty() {
                lines.push(String::new());
            }
            lines.push("  [dev-dependencies]".to_string());
            for (n, v) in &self.dev_dependencies {
                lines.push(format!("    {} = \"{}\"", n, v));
            }
        }
        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flat_fields() {
        let toml_str = r#"
name = "testproj"
version = "0.1.0"
edition = "2024"
authors = ["Test Author"]
license = "MIT"
description = "A test project"

[dependencies]
foo = "1.0.0"
"#;
        let manifest: Manifest = toml::from_str(toml_str).unwrap();
        assert_eq!(manifest.project_name(), "testproj");
        assert_eq!(manifest.project_version(), "0.1.0");
        assert_eq!(manifest.project_edition(), "2024");
        assert_eq!(manifest.dependencies.len(), 1);
    }

    #[test]
    fn test_project_table() {
        let toml_str = r#"
[project]
name = "myapp"
version = "1.0.0"
edition = "2024"
authors = ["Me"]
license = "MIT"
description = "My app"
main = "src/main.ky"

[dependencies]
math = "1.0.0"
"#;
        let manifest: Manifest = toml::from_str(toml_str).unwrap();
        assert_eq!(manifest.project_name(), "myapp");
        assert_eq!(manifest.project_version(), "1.0.0");
        assert_eq!(manifest.project_main(), "src/main.ky");
        assert!(manifest.project.is_some());
    }

    #[test]
    fn test_project_table_overrides_flat() {
        let toml_str = r#"
name = "oldname"
version = "0.0.1"

[project]
name = "newname"
version = "2.0.0"

[dependencies]
"#;
        let manifest: Manifest = toml::from_str(toml_str).unwrap();
        assert_eq!(manifest.project_name(), "newname");
        assert_eq!(manifest.project_version(), "2.0.0");
    }

    #[test]
    fn test_validate_valid() {
        let toml_str = r#"
name = "valid"
version = "1.0.0"
edition = "2024"

[dependencies]
"#;
        let manifest: Manifest = toml::from_str(toml_str).unwrap();
        assert!(manifest.validate().is_ok());
    }

    #[test]
    fn test_validate_invalid_name() {
        let toml_str = r#"
name = ""
version = "1.0.0"

[dependencies]
"#;
        let manifest: Manifest = toml::from_str(toml_str).unwrap();
        assert!(manifest.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_semver() {
        let toml_str = r#"
name = "test"
version = "not-a-version"

[dependencies]
"#;
        let manifest: Manifest = toml::from_str(toml_str).unwrap();
        assert!(manifest.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_dep_version() {
        let toml_str = r#"
name = "test"
version = "1.0.0"

[dependencies]
bad = "not-valid"
"#;
        let manifest: Manifest = toml::from_str(toml_str).unwrap();
        assert!(manifest.validate().is_err());
    }

    #[test]
    fn test_dev_dependencies_parsed() {
        let toml_str = r#"
name = "test"
version = "1.0.0"

[dependencies]
foo = "1.0.0"

[dev-dependencies]
bar = "2.0.0"
"#;
        let manifest: Manifest = toml::from_str(toml_str).unwrap();
        assert_eq!(manifest.dev_dependencies.len(), 1);
        assert_eq!(manifest.dev_dependencies.get("bar").unwrap(), "2.0.0");
    }

    #[test]
    fn test_minimal_manifest() {
        let toml_str = r#"
name = "minimal"
version = "0.1.0"
"#;
        let manifest: Manifest = toml::from_str(toml_str).unwrap();
        assert_eq!(manifest.project_name(), "minimal");
        assert!(manifest.validate().is_ok());
        assert!(manifest.dependencies.is_empty());
    }

    #[test]
    fn test_dependency_summary() {
        let toml_str = r#"
name = "test"
version = "1.0.0"

[dependencies]
foo = "1.0.0"
"#;
        let manifest: Manifest = toml::from_str(toml_str).unwrap();
        let summary = manifest.dependency_summary();
        assert!(summary.iter().any(|l| l.contains("foo")));
    }
}
