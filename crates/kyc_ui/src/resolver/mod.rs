use std::path::{Path, PathBuf};
use std::fs;

use crate::ast::KyxFile;

/// Resolve a component tag name to a file path and parse it.
/// Search order:
///  1. `./views/<name>.kyx`
///  2. `./components/<name>.kyx`
///  3. `./<name>.kyx`
///  4. `src/views/<name>.kyx`
///  5. `src/components/<name>.kyx`
///  6. `src/<name>.kyx`
///  7. `packages/<name>/src/lib.kyx`
pub fn resolve_component(tag_name: &str, base_dir: &Path) -> Result<(PathBuf, String), String> {
    let searches = build_search_paths(tag_name, base_dir);

    for path in &searches {
        if path.exists() {
            let content = fs::read_to_string(path)
                .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
            return Ok((path.clone(), content));
        }
    }

    Err(format!(
        "Component '{}' not found. Searched:\n  {}",
        tag_name,
        searches.iter().map(|p| p.display().to_string()).collect::<Vec<_>>().join("\n  ")
    ))
}

/// Build search paths for a component tag name.
fn build_search_paths(tag_name: &str, base_dir: &Path) -> Vec<PathBuf> {
    let base = if base_dir.is_absolute() {
        base_dir.to_path_buf()
    } else {
        std::env::current_dir().unwrap_or_default().join(base_dir)
    };

    // Convert tag name: "home_view" -> "home", "user_profile" -> "user_profile"
    let name = tag_name.to_lowercase();
    // Try direct name and also strip known suffixes
    let names: Vec<String> = {
        let mut n = vec![name.clone()];
        // "home_view" -> also try "home"
        if let Some(stripped) = name.strip_suffix("_view") {
            n.push(stripped.to_string());
        }
        if let Some(stripped) = name.strip_suffix("_page") {
            n.push(stripped.to_string());
        }
        if let Some(stripped) = name.strip_suffix("_component") {
            n.push(stripped.to_string());
        }
        if let Some(stripped) = name.strip_suffix("_layout") {
            n.push(stripped.to_string());
        }
        n
    };

    let mut paths = Vec::new();
    for n in &names {
        let kyx = format!("{}.kyx", n);
        // Priority: src/views, src/layouts, src/components first (new structure)
        paths.push(base.join("src").join("views").join(&kyx));
        paths.push(base.join("src").join("layouts").join(&kyx));
        paths.push(base.join("src").join("components").join(&kyx));
        paths.push(base.join("src").join(&kyx));
        // Fallback: root-level dirs (old structure)
        paths.push(base.join("views").join(&kyx));
        paths.push(base.join("layouts").join(&kyx));
        paths.push(base.join("components").join(&kyx));
        paths.push(base.join(&kyx));
    }

    paths
}

/// Get the custom component tag names from a parsed KyxFile.
/// Extracts from body elements, custom tags, and <route> element attributes.
pub fn extract_custom_tags(file: &crate::ast::KyxFile) -> Vec<String> {
    let mut tags = Vec::new();
    extract_from_nodes(&file.body, &mut tags);
    tags.sort();
    tags.dedup();
    tags
}

fn extract_from_nodes(nodes: &[crate::ast::KyxNode], tags: &mut Vec<String>) {
    use crate::ast::{AttrValue, KyxNode};
    for node in nodes {
        match node {
            KyxNode::Element { tag, children, .. } => {
                // If this is a <route> element, extract component and layout attrs
                if tag.to_lowercase() == "route" {
                    extract_route_attrs_from_element(node, tags);
                } else {
                    check_custom_tag(tag, tags);
                }
                extract_from_nodes(children, tags);
            }
            KyxNode::SelfClosing { tag, attrs } => {
                if tag.to_lowercase() == "route" {
                    // Extract component and layout from route attrs
                    for a in attrs {
                        if a.name == "component" {
                            let val = attr_value(&a.value);
                            if !val.is_empty() { check_custom_tag(&val, tags); }
                        }
                        if a.name == "layout" {
                            let val = attr_value(&a.value);
                            if !val.is_empty() { check_custom_tag(&val, tags); }
                        }
                    }
                } else {
                    check_custom_tag(tag, tags);
                }
            }
            KyxNode::Slot { fallback, .. } => extract_from_nodes(fallback, tags),
            KyxNode::If { then_branch, else_branch, .. } => {
                extract_from_nodes(then_branch, tags);
                extract_from_nodes(else_branch, tags);
            }
            KyxNode::For { body, .. } => extract_from_nodes(body, tags),
            KyxNode::Match { cases, .. } => {
                for c in cases {
                    extract_from_nodes(&c.body, tags);
                }
            }
            _ => {}
        }
    }
}

/// Extract component and layout attributes from a <route> Element node
fn extract_route_attrs_from_element(node: &crate::ast::KyxNode, tags: &mut Vec<String>) {
    use crate::ast::{AttrValue, KyxNode};
    if let KyxNode::Element { attrs, .. } = node {
        for a in attrs {
            if a.name == "component" {
                let val = attr_value(&a.value);
                if !val.is_empty() { check_custom_tag(&val, tags); }
            }
            if a.name == "layout" {
                let val = attr_value(&a.value);
                if !val.is_empty() { check_custom_tag(&val, tags); }
            }
        }
    }
}

fn attr_value(val: &crate::ast::AttrValue) -> String {
    match val {
        crate::ast::AttrValue::String(s) => s.clone(),
        crate::ast::AttrValue::Expr(e) => e.trim_start_matches('@').to_string(),
        crate::ast::AttrValue::Flag => String::new(),
    }
}

fn check_custom_tag(tag: &str, tags: &mut Vec<String>) {
    if crate::ir::ComponentTag::from_str(tag) == crate::ir::ComponentTag::Custom(tag.to_string()) {
        if !tags.contains(&tag.to_string()) {
            tags.push(tag.to_string());
        }
    }
}

/// Resolve a single component file, parse it, and return the KyxFile.
pub fn resolve_and_parse(tag_name: &str, base_dir: &Path) -> Result<(PathBuf, KyxFile), String> {
    let (path, content) = resolve_component(tag_name, base_dir)?;
    let file = crate::parser::parse(&content)
        .map_err(|e| format!("Error parsing {}: {}", path.display(), e))?;
    Ok((path, file))
}

/// Resolve all custom component tags from a KyxFile.
/// Returns a list of (tag_name, file_path, KyxFile) for each resolved dependency.
/// The entry file itself is NOT included.
pub fn resolve_all_components(
    file: &KyxFile,
    base_dir: &Path,
) -> Result<Vec<(String, PathBuf, KyxFile)>, String> {
    let tags = extract_custom_tags(file);
    let mut resolved = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for tag in &tags {
        if seen.contains(tag) { continue; }
        seen.insert(tag.clone());

        if let Ok((path, dep_file)) = resolve_and_parse(tag, base_dir) {
            resolved.push((tag.clone(), path, dep_file));
        }
    }

    Ok(resolved)
}

/// Resolve an import module path to a file path.
/// "views.home" → tries src/views/home.kyx, views/home.kyx, etc.
fn resolve_import_module(module: &str, base_dir: &Path) -> Result<PathBuf, String> {
    let file_stem = module.replace('.', "/");
    let base = if base_dir.is_absolute() {
        base_dir.to_path_buf()
    } else {
        std::env::current_dir().unwrap_or_default().join(base_dir)
    };
    // Try paths in priority order
    let candidates = vec![
        base.join("src").join(&file_stem).with_extension("kyx"),
        base.join("src").join(&file_stem).with_extension("ky"),
        base.join(&file_stem).with_extension("kyx"),
        base.join(&file_stem).with_extension("ky"),
    ];
    for p in &candidates {
        if p.exists() {
            return Ok(p.clone());
        }
    }
    Err(format!("Import '{}' not found. Searched: {:?}", module,
        candidates.iter().map(|p| p.display().to_string()).collect::<Vec<_>>()))
}

/// Build a multi-file UiProgram from an entry .kyx file and its dependencies.
/// 1. Processes explicit `from X import Y` declarations
/// 2. Auto-resolves any remaining custom component tags as fallback
pub fn build_multifile_program(
    source: &str,
    source_path: &Path,
) -> Result<crate::ir::UiProgram, String> {
    use crate::ir::{ComponentRenderer, UiProgram};

    // Parse entry file
    let entry_file = crate::parser::parse(source)
        .map_err(|e| format!("Parse error in entry file: {}", e))?;

    let base_dir = source_path.parent().unwrap_or(Path::new("."));

    // 1. Process explicit imports
    let mut component_renderers = Vec::new();
    let mut resolved_names = std::collections::HashSet::new();

    for imp in &entry_file.imports {
        let path = resolve_import_module(&imp.module, base_dir)?;
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
        let dep_file = crate::parser::parse(&content)
            .map_err(|e| format!("Error parsing {}: {}", path.display(), e))?;
        let dep_program = crate::parser::to_ui_program(dep_file);
        component_renderers.push(ComponentRenderer {
            name: imp.name.clone(),
            code_blocks: dep_program.code_blocks,
            styles: dep_program.styles,
            animations: dep_program.animations,
            body: dep_program.body,
        });
        resolved_names.insert(imp.name.clone());
    }

    // 2. Auto-resolve any custom tags not covered by imports (fallback)
    let custom_tags = extract_custom_tags(&entry_file);
    for tag in &custom_tags {
        if resolved_names.contains(tag) { continue; }
        if let Ok((_path, dep_file)) = resolve_and_parse(tag, base_dir) {
            let dep_program = crate::parser::to_ui_program(dep_file);
            component_renderers.push(ComponentRenderer {
                name: tag.clone(),
                code_blocks: dep_program.code_blocks,
                styles: dep_program.styles,
                animations: dep_program.animations,
                body: dep_program.body,
            });
            resolved_names.insert(tag.clone());
        }
    }

    // Convert entry to IR
    let entry_program = crate::parser::to_ui_program(entry_file);
    let mut all_routes = entry_program.routes;
    let mut all_styles = entry_program.styles;
    let mut all_animations = entry_program.animations;

    // Collect styles/animations from all renderers too
    for comp in &component_renderers {
        all_styles.extend(comp.styles.clone());
        all_animations.extend(comp.animations.clone());
    }

    // De-duplicate routes by path (last one wins)
    let mut route_map: std::collections::HashMap<String, crate::ir::RouteConfig> = std::collections::HashMap::new();
    for r in all_routes {
        route_map.insert(r.path.clone(), r);
    }

    Ok(UiProgram {
        routes: route_map.into_values().collect(),
        code_blocks: entry_program.code_blocks,
        styles: all_styles,
        animations: all_animations,
        body: entry_program.body,
        component_renderers,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_build_search_paths() {
        let base = Path::new("/project");
        let paths = build_search_paths("home_view", base);

        // Check that we search views and components dirs
        let has_views = paths.iter().any(|p| p.to_str().unwrap_or("").contains("views/home.kyx"));
        let has_components = paths.iter().any(|p| p.to_str().unwrap_or("").contains("components/home.kyx"));
        let has_direct = paths.iter().any(|p| p.to_str().unwrap_or("").ends_with("home.kyx"));

        assert!(has_views, "Should search views/home.kyx");
        assert!(has_components, "Should search components/home.kyx");
        assert!(has_direct, "Should search ./home.kyx");
    }

    #[test]
    fn test_name_suffix_stripping() {
        let base = Path::new("/project");
        // home_view should also try "home"
        let paths = build_search_paths("home_view", base);
        let has_home = paths.iter().any(|p| p.to_str().unwrap_or("").contains("/home.kyx"));
        let has_home_view = paths.iter().any(|p| p.to_str().unwrap_or("").contains("/home_view.kyx"));
        assert!(has_home, "Should try home.kyx (stripped suffix)");
        assert!(has_home_view, "Should also try home_view.kyx (full name)");
    }

    #[test]
    fn test_resolve_nonexistent() {
        let base = Path::new("/nonexistent_dir_xyz");
        let result = resolve_component("does_not_exist_abc", base);
        assert!(result.is_err(), "Should fail for nonexistent component");
    }
}
