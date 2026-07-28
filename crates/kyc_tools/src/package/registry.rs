use kyc_core::resolver::{DepSpec, RegistryBackend};
use kyc_core::semver::{parse_version, parse_requirement, version_to_string, SemanticVersion};
use serde::Deserialize;
use std::collections::HashMap;

const DEFAULT_REGISTRY: &str = "https://IT-KYNERA.github.io/KYLE/docs";
const REGISTRY_ENV: &str = "KL_REGISTRY";

/// Package version entry from the registry API.
#[derive(Debug, Deserialize)]
struct RegistryVersionEntry {
    version: String,
    #[allow(dead_code)]
    yanked: Option<bool>,
}

/// Package detail response from registry API.
#[derive(Debug, Deserialize)]
struct RegistryPackageResponse {
    versions: Vec<RegistryVersionEntry>,
}

/// Dependency entry in a package's ky.toml from registry.
#[derive(Debug, Deserialize)]
struct RegistryDepEntry {
    name: String,
    version: String,
}

/// Package metadata from registry (for getting deps of a specific version).
#[derive(Debug, Deserialize)]
struct RegistryVersionDetail {
    dependencies: Vec<RegistryDepEntry>,
}

/// HTTP+JSON registry client implementing RegistryBackend.
pub struct RegistryClient {
    registry_url: String,
    /// In-memory cache of version lists (avoids repeated HTTP calls).
    version_cache: HashMap<String, Vec<SemanticVersion>>,
    /// In-memory cache of dependency lists.
    #[allow(dead_code)]
    dep_cache: HashMap<(String, SemanticVersion), Vec<DepSpec>>,
}

impl RegistryClient {
    pub fn new() -> Self {
        let url = std::env::var(REGISTRY_ENV).unwrap_or_else(|_| DEFAULT_REGISTRY.to_string());
        Self {
            registry_url: url,
            version_cache: HashMap::new(),
            dep_cache: HashMap::new(),
        }
    }

    #[allow(dead_code)]
    pub fn with_url(url: &str) -> Self {
        Self {
            registry_url: url.to_string(),
            version_cache: HashMap::new(),
            dep_cache: HashMap::new(),
        }
    }

    fn fetch_versions_from_api(&self, name: &str) -> Result<Vec<SemanticVersion>, String> {
        let url = format!("{}/packages/{}.json", self.registry_url, name);
        let response = ureq::get(&url)
            .call()
            .map_err(|e| format!("Failed to fetch package '{}': {}", name, e))?;

        if response.status() != 200 {
            return Err(format!(
                "Registry returned {} for package '{}'",
                response.status(),
                name
            ));
        }

        let body_bytes = response.into_body().read_to_vec()
            .map_err(|e| format!("Failed to read registry response: {}", e))?;

        let body: RegistryPackageResponse = serde_json::from_slice(&body_bytes)
            .map_err(|e| format!("Failed to parse registry response: {}", e))?;

        let mut versions: Vec<SemanticVersion> = body.versions
            .into_iter()
            .filter(|v| !v.yanked.unwrap_or(false))
            .filter_map(|v| parse_version(&v.version).ok())
            .collect();

        versions.sort();
        Ok(versions)
    }

    fn fetch_deps_from_api(
        &self,
        name: &str,
        version: &SemanticVersion,
    ) -> Result<Vec<DepSpec>, String> {
        let ver_str = version_to_string(version);
        let url = format!("{}/packages/{}/{}/deps.json", self.registry_url, name, ver_str);

        let response = ureq::get(&url)
            .call()
            .map_err(|e| format!("Failed to fetch deps for '{}@{}': {}", name, ver_str, e))?;

        if response.status() == 404 {
            return Ok(Vec::new());
        }

        if response.status() != 200 {
            return Err(format!(
                "Registry returned {} for deps of '{}@{}'",
                response.status(),
                name,
                ver_str
            ));
        }

        let body_bytes = response.into_body().read_to_vec()
            .map_err(|e| format!("Failed to read dependency response: {}", e))?;

        let body: RegistryVersionDetail = serde_json::from_slice(&body_bytes)
            .map_err(|e| format!("Failed to parse dependency response: {}", e))?;

        Ok(body.dependencies
            .into_iter()
            .filter_map(|d| {
                parse_requirement(&d.version).ok().map(|req| DepSpec {
                    name: d.name,
                    requirement: req,
                })
            })
            .collect())
    }
}

impl RegistryBackend for RegistryClient {
    fn get_versions(&self, name: &str) -> Result<Vec<SemanticVersion>, String> {
        if let Some(cached) = self.version_cache.get(name) {
            return Ok(cached.clone());
        }
        let versions = self.fetch_versions_from_api(name)?;
        Ok(versions)
    }

    fn get_dependencies(
        &self,
        name: &str,
        version: &SemanticVersion,
    ) -> Result<Vec<DepSpec>, String> {
        self.fetch_deps_from_api(name, version)
    }
}

/// File-based registry for local testing/development.
/// Reads packages from a local directory tree.
pub struct FileRegistry {
    base_path: std::path::PathBuf,
}

impl FileRegistry {
    pub fn new(path: &std::path::Path) -> Self {
        Self {
            base_path: path.to_path_buf(),
        }
    }

    #[allow(dead_code)]
    fn package_dir(&self, name: &str) -> std::path::PathBuf {
        self.base_path.join(name)
    }

    fn read_manifest(&self, name: &str, version: &str) -> Result<crate::package::Manifest, String> {
        let path = self.package_dir(name).join(version).join("ky.toml");
        crate::package::Manifest::read(&path)
    }
}

impl RegistryBackend for FileRegistry {
    fn get_versions(&self, name: &str) -> Result<Vec<SemanticVersion>, String> {
        let dir = self.package_dir(name);
        if !dir.exists() {
            return Ok(Vec::new());
        }

        let mut versions: Vec<SemanticVersion> = Vec::new();
        for entry in std::fs::read_dir(&dir)
            .map_err(|e| format!("Failed to read {}: {}", dir.display(), e))?
        {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let dir_name = entry.file_name().to_string_lossy().to_string();
            if entry.path().is_dir() && entry.path().join("ky.toml").exists() {
                if let Ok(ver) = parse_version(&dir_name) {
                    versions.push(ver);
                }
            }
        }

        versions.sort();
        Ok(versions)
    }

    fn get_dependencies(
        &self,
        name: &str,
        version: &SemanticVersion,
    ) -> Result<Vec<DepSpec>, String> {
        let ver_str = version_to_string(version);
        let manifest = self.read_manifest(name, &ver_str)?;

        Ok(manifest.dependencies
            .into_iter()
            .filter_map(|(n, r)| {
                parse_requirement(&r).ok().map(|req| DepSpec {
                    name: n,
                    requirement: req,
                })
            })
            .collect())
    }
}

/// Attempt to download a package tarball from the registry.
pub fn download_package(name: &str, version: &str) -> Result<Vec<u8>, String> {
    let registry_url = std::env::var(REGISTRY_ENV).unwrap_or_else(|_| DEFAULT_REGISTRY.to_string());

    // Support file:// URLs for local development
    if let Some(path) = registry_url.strip_prefix("file://") {
        let tarball_path = std::path::Path::new(path).join(name).join(format!("{}.tar.gz", version));
        return std::fs::read(&tarball_path)
            .map_err(|e| format!("Failed to read package file '{}': {}", tarball_path.display(), e));
    }

    let url = format!("{}/packages/{}/{}/download.tar.gz", registry_url, name, version);

    let response = ureq::get(&url)
        .call()
        .map_err(|e| format!("Failed to download '{}@{}': {}", name, version, e))?;

    if response.status() != 200 {
        return Err(format!(
            "Registry returned {} when downloading '{}@{}'",
            response.status(),
            name,
            version
        ));
    }

    let body = response.into_body().read_to_vec()
        .map_err(|e| format!("Failed to read download: {}", e))?;

    Ok(body)
}

/// Fetch a package's manifest from the registry (for resolving deps without downloading full tarball).
#[allow(dead_code)]
pub fn fetch_package_manifest(name: &str, version: &str) -> Result<String, String> {
    let registry_url = std::env::var(REGISTRY_ENV).unwrap_or_else(|_| DEFAULT_REGISTRY.to_string());
    let url = format!("{}/packages/{}/{}/ky.toml", registry_url, name, version);

    let response = ureq::get(&url)
        .call()
        .map_err(|e| format!("Failed to fetch manifest for '{}@{}': {}", name, version, e))?;

    if response.status() != 200 {
        return Err(format!(
            "Registry returned {} for manifest of '{}@{}'",
            response.status(),
            name,
            version
        ));
    }

    let body_bytes = response.into_body().read_to_vec()
        .map_err(|e| format!("Failed to read manifest: {}", e))?;

    Ok(String::from_utf8(body_bytes)
        .map_err(|e| format!("Invalid UTF-8 in manifest: {}", e))?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_client_creation() {
        let client = RegistryClient::new();
        assert!(client.registry_url.contains("IT-KYNERA.github.io"));
    }

    #[test]
    fn test_file_registry_no_package() {
        let dir = std::env::temp_dir().join("kl_reg_test_nonexistent");
        let reg = FileRegistry::new(&dir);
        let versions = reg.get_versions("nonexistent").unwrap();
        assert!(versions.is_empty());
    }

    #[test]
    fn test_download_no_registry() {
        let result = download_package("test-pkg", "1.0.0");
        assert!(result.is_err());
    }
}
