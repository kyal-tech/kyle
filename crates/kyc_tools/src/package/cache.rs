use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

const CACHE_DIR: &str = ".ky/cache";
const KY_HOME_ENV: &str = "KY_HOME";

/// Returns the cache root directory (~/.ky/cache/ or $KY_HOME/cache/).
pub fn cache_root() -> PathBuf {
    if let Ok(home) = std::env::var(KY_HOME_ENV) {
        PathBuf::from(home).join("cache")
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(CACHE_DIR)
    } else {
        PathBuf::from(".ky").join("cache")
    }
}

/// Returns the path for a cached package: ~/.ky/cache/<name>-<version>/
pub fn package_cache_dir(name: &str, version: &str) -> PathBuf {
    cache_root().join(format!("{}-{}", name, version))
}

/// Returns the path to the manifest of a cached package.
pub fn package_manifest_path(name: &str, version: &str) -> PathBuf {
    package_cache_dir(name, version).join("ky.toml")
}

/// Returns the source directory of a cached package.
pub fn package_src_dir(name: &str, version: &str) -> PathBuf {
    package_cache_dir(name, version).join("src")
}

/// Check if a package version is already cached.
pub fn is_cached(name: &str, version: &str) -> bool {
    let dir = package_cache_dir(name, version);
    dir.exists() && dir.join("ky.toml").exists()
}

/// Compute SHA256 checksum of a file.
pub fn sha256_checksum(path: &Path) -> Result<String, String> {
    let data = fs::read(path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    let mut hasher = Sha256::new();
    hasher.update(&data);
    Ok(format!("{:x}", hasher.finalize()))
}

/// Ensure the cache directory exists.
pub fn ensure_cache_dir() -> Result<PathBuf, String> {
    let root = cache_root();
    fs::create_dir_all(&root)
        .map_err(|e| format!("Failed to create cache dir {}: {}", root.display(), e))?;
    Ok(root)
}

/// Extract a .tar.gz archive into the cache directory.
pub fn extract_tarball(tarball_data: &[u8], dest: &Path) -> Result<(), String> {
    fs::create_dir_all(dest)
        .map_err(|e| format!("Failed to create {}: {}", dest.display(), e))?;

    let decoder = flate2::read::GzDecoder::new(tarball_data);
    let mut archive = tar::Archive::new(decoder);

    archive.unpack(dest)
        .map_err(|e| format!("Failed to extract tarball: {}", e))?;

    Ok(())
}

/// List all cached packages (name-version directories).
pub fn list_cached_packages() -> Result<Vec<(String, String)>, String> {
    let root = cache_root();
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut packages = Vec::new();
    for entry in fs::read_dir(&root)
        .map_err(|e| format!("Failed to read cache dir: {}", e))?
    {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let dir_name = entry.file_name().to_string_lossy().to_string();

        // Format: <name>-<version>
        if let Some(sep_pos) = dir_name.rfind('-') {
            let name = &dir_name[..sep_pos];
            let version = &dir_name[sep_pos + 1..];
            if entry.path().is_dir() && entry.path().join("ky.toml").exists() {
                packages.push((name.to_string(), version.to_string()));
            }
        }
    }

    packages.sort();
    Ok(packages)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_cache_root() {
        let root = cache_root();
        assert!(root.to_string_lossy().contains("cache"));
    }

    #[test]
    fn test_package_cache_dir_format() {
        let dir = package_cache_dir("mylib", "1.2.3");
        let path_str = dir.to_string_lossy();
        assert!(path_str.contains("mylib-1.2.3"));
    }

    #[test]
    fn test_is_cached_returns_false_for_nonexistent() {
        assert!(!is_cached("nonexistent-pkg-12345", "9.9.9"));
    }

    #[test]
    fn test_sha256_checksum() {
        let dir = std::env::temp_dir().join("kl_cache_test_sha256");
        let _ = fs::create_dir_all(&dir);
        let file_path = dir.join("test.txt");
        fs::write(&file_path, b"hello world").unwrap();

        let hash = sha256_checksum(&file_path).unwrap();
        // SHA256 of "hello world"
        assert_eq!(hash, "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_list_cached_packages_empty() {
        let pkgs = list_cached_packages().unwrap_or_default();
        // May or may not have packages, but should not crash
        assert!(pkgs.iter().all(|(n, v)| !n.is_empty() && !v.is_empty()));
    }
}
