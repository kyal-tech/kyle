// kyc_semantic::module_resolver — Module resolution for imports
//
// Resolves `import X`, `from X import Y`, and relative `~X` imports
// by finding and parsing the corresponding .ky files.
//
// Module path resolution rules:
//   - Dots in module names map to directory separators:
//     `import utils.helpers` → `utils/helpers.kl`
//   - Relative `~` prefix resolves relative to the source file's directory
//   - Absolute search resolves against search_paths
//   - Public declarations (no `_`/`__` prefix) are returned for `import X`
//   - Explicit `from X import Y` can import public or protected declarations

use std::path::PathBuf;
use std::fs;
use std::collections::HashMap;
use kyc_core::ast::{Decl, Program};
use kyc_frontend::lexer::Lexer;
use kyc_frontend::parser::Parser;

/// A resolved module with its declarations.
pub struct ResolvedModule {
    pub name: String,
    pub program: Program,
    pub path: PathBuf,
}

/// Resolves import declarations by finding and parsing .ky files.
pub struct ModuleResolver {
    /// Search paths for module resolution (used by absolute imports).
    pub search_paths: Vec<PathBuf>,
    /// Cache of already resolved modules (name -> module).
    pub cache: HashMap<String, ResolvedModule>,
    /// Directory of the source file being compiled (for relative `~` imports).
    pub source_dir: Option<PathBuf>,
}

impl ModuleResolver {
    pub fn new() -> Self {
        Self {
            search_paths: Vec::new(),
            cache: HashMap::new(),
            source_dir: None,
        }
    }

    /// Set the source file directory for relative imports.
    pub fn set_source_dir(&mut self, dir: PathBuf) {
        self.source_dir = Some(dir);
    }

    /// Add a directory to search for modules.
    pub fn add_search_path(&mut self, path: PathBuf) {
        if !self.search_paths.contains(&path) {
            self.search_paths.push(path);
        }
    }

    /// Resolve an `import X` declaration.
    /// `relative` indicates whether this is a `~` relative import.
    /// Returns the parsed module and its declarations.
    pub fn resolve_import(&mut self, module_name: &str, relative: bool) -> Result<&ResolvedModule, String> {
        // Build a unique cache key that includes whether it was relative
        let cache_key = format!("{}{}", if relative { "~" } else { "" }, module_name);
        if self.cache.contains_key(&cache_key) {
            return Ok(self.cache.get(&cache_key).unwrap());
        }

        let path = self.find_module_file(module_name, relative)?;
        let source = fs::read_to_string(&path)
            .map_err(|e| format!("cannot read module '{}': {}", module_name, e))?;

        let mut lexer = Lexer::new(&source);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let program = parser.parse()
            .map_err(|e| format!("parse error in module '{}': {}", module_name, e))?;

        let module = ResolvedModule {
            name: module_name.to_string(),
            program,
            path,
        };
        self.cache.insert(cache_key.clone(), module);
        Ok(self.cache.get(&cache_key).unwrap())
    }

    /// Find a module file, using dotted path resolution.
    /// - `relative=true`: resolve relative to source_dir
    /// - `relative=false`: resolve against search_paths
    /// Dots in module_name are converted to directory separators.
    fn find_module_file(&self, module_name: &str, relative: bool) -> Result<PathBuf, String> {
        // Convert dots to directory separators
        let path_str = module_name.replace('.', "/");
        let file_name = format!("{}.ky", path_str);
        let file_name_kyx = format!("{}.kyx", path_str);

        if relative {
            // Resolve relative to source file directory
            if let Some(ref source_dir) = self.source_dir {
                for fname in [&file_name, &file_name_kyx] {
                    let candidate = source_dir.join(fname);
                    if candidate.exists() {
                        return Ok(candidate);
                    }
                }
                return Err(format!("relative module '{}' not found at {:?}", module_name, source_dir.join(&file_name)));
            }
            return Err("no source directory set for relative import".to_string());
        }

        // Resolve against search paths
        for search_path in &self.search_paths {
            // Try <name>.ky and <name>.kyx
            for fname in [&file_name, &file_name_kyx] {
                let candidate = search_path.join(fname);
                if candidate.exists() {
                    return Ok(candidate);
                }
            }
            // Try <name>/lib.ky and <name>/lib.kyx for directory-based packages
            for lib_ext in ["ky", "kyx"] {
                let dir_candidate = search_path.join(&path_str).join(format!("lib.{}", lib_ext));
                if dir_candidate.exists() {
                    return Ok(dir_candidate);
                }
                // Try packages/<name>/src/lib.ky and lib.kyx
                let src_candidate = search_path.join(&path_str).join("src").join(format!("lib.{}", lib_ext));
                if src_candidate.exists() {
                    return Ok(src_candidate);
                }
            }
            // Try packages/<name>/src/<file> for submodules (e.g. http/server.ky → http/src/server.ky)
            if path_str.contains('/') {
                if let Some(parent) = PathBuf::from(&path_str).parent() {
                    if let Some(file) = PathBuf::from(&path_str).file_name() {
                        for ext in ["ky", "kyx"] {
                            let src_candidate = search_path.join(parent).join("src").join(file).with_extension(ext);
                            if src_candidate.exists() {
                                return Ok(src_candidate);
                            }
                        }
                    }
                }
                // Fallback: try <search_path>/<path>/lib.ky (directory-based package with nested path)
                for ext in ["ky", "kyx"] {
                    let dir_candidate = search_path.join(&path_str).join(format!("lib.{}", ext));
                    if dir_candidate.exists() {
                        return Ok(dir_candidate);
                    }
                }
            }
        }
        Err(format!("module '{}' not found in search paths", module_name))
    }

    /// Check if a declaration name is public (no `_` or `__` prefix).
    fn is_public(name: &str) -> bool {
        !name.starts_with('_')
    }

    /// Check if a declaration name is at least protected (not `__` private).
    fn is_not_private(name: &str) -> bool {
        !name.starts_with("__")
    }

    /// Get public declarations from a module (for `import X` — only public names).
    /// Private (`__`) and protected (`_`) names are excluded.
    pub fn get_module_declarations(&mut self, module_name: &str, relative: bool) -> Result<Vec<Decl>, String> {
        let module = self.resolve_import(module_name, relative)?;
        let decls: Vec<Decl> = module.program.declarations.iter()
            .filter(|decl| {
                let name = match decl {
                    Decl::Function(f) => &f.name,
                    Decl::Variable(v) => &v.name,
                    Decl::Constant(c) => &c.name,
                    Decl::Class(c) => &c.name,
                    Decl::AbstractClass(c) => &c.name,
                    Decl::Struct(s) => &s.name,
                    Decl::Enum(e) => &e.name,
                    Decl::Contract(c) => &c.name,
                    Decl::TypeAlias(t) => &t.name,
                    Decl::Link(_, _) => return false,
                    Decl::Use(_) | Decl::Import(_) | Decl::FromImport(_) => return false,
                    Decl::Expression(_) => return false,
                };
                Self::is_public(name)
            })
            .cloned()
            .collect();
        Ok(decls)
    }

    /// Get a specific declaration from a module (for `from X import Y`).
    /// Allows public and protected names, but not private (`__`) names.
    pub fn get_imported_declaration(&mut self, module_name: &str, name: &str, relative: bool) -> Result<Decl, String> {
        let module = self.resolve_import(module_name, relative)?;
        for decl in &module.program.declarations {
            let decl_name = match decl {
                Decl::Function(f) => &f.name,
                Decl::Variable(v) => &v.name,
                Decl::Constant(c) => &c.name,
                Decl::Class(c) => &c.name,
                Decl::AbstractClass(c) => &c.name,
                Decl::Struct(s) => &s.name,
                Decl::Enum(e) => &e.name,
                Decl::Contract(c) => &c.name,
                Decl::TypeAlias(t) => &t.name,
                Decl::Link(_, _) => continue,
                Decl::Use(_) | Decl::Import(_) | Decl::FromImport(_) => continue,
                Decl::Expression(_) => continue,
            };
            if decl_name == name {
                if !Self::is_not_private(name) {
                    return Err(format!("'{}' is private and cannot be imported", name));
                }
                return Ok(decl.clone());
            }
        }
        Err(format!("'{}' not found in module '{}'", name, module_name))
    }
}

impl Default for ModuleResolver {
    fn default() -> Self {
        Self::new()
    }
}
