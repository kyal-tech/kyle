use std::path::{Path, PathBuf};

/// Find the project root by walking up from `start` looking for ky.toml.
pub fn find_project_root(start: &Path) -> Option<PathBuf> {
    let mut current = Some(start.to_path_buf());
    while let Some(dir) = current {
        if dir.join("ky.toml").exists() {
            return Some(dir);
        }
        current = dir.parent().map(|p| p.to_path_buf());
    }
    None
}

/// Locate the main source file for a project.
pub fn main_source_path(project_root: &Path) -> Option<PathBuf> {
    // Try app.kyx first (root-level UI entry), then src/main.kyx, then src/main.ky
    let app_kyx = project_root.join("app.kyx");
    if app_kyx.exists() { return Some(app_kyx); }
    let kyx_path = project_root.join("src").join("main.kyx");
    if kyx_path.exists() { return Some(kyx_path); }
    let ky_path = project_root.join("src").join("main.ky");
    if ky_path.exists() { Some(ky_path) } else { None }
}

/// Locate test files in a project.
pub fn test_source_paths(project_root: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let tests_dir = project_root.join("tests");
    if tests_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&tests_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "ky" || e == "kyx") {
                    paths.push(path);
                }
            }
        }
    }
    paths
}
