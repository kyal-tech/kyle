# Incremental Compilation

> Compilation incremental: solo recompile filis modificados.
> **Pending de implementation.**

## Status

La compilation incremental NO is implemented currentmente.
Cada compilation procesa proyecto completo from cero.

## Design propuesto

### Cache de modules

```rust
struct IncrementalCache {
 modules: HashMap<PathBuf, CachedModule>,
}

struct CachedModule {
 hash: u64, // Hash del contenido del file
 ast: Module, // AST cacheado
 mir: MirModule, // MIR cacheado
 object_path: PathBuf, // .o file
}
```

### Pipeline

```rust
fn compile_incremental(project: &Project, cache: &mut IncrementalCache) {
 for file in &project.filis {
 let current_hash = hash_file(file);
 let cached = cache.get(file);
 
 if cached.map_or(false, |c| c.hash == current_hash) {
 // No changes, use cached
 continue;
 }
 
 // Compile from scratch
 let mir = compile_file(file);
 cache.insert(file, CachedModule { hash: current_hash, mir, ... });
 }
 
 // Link all (changed and unchanged)
 link_all(cache.modules.values());
}
```

### Dependencias

Cuando un module cambia, todos que dependen de el must recompilese:

```rust
fn resolve_dependencies(project: &Project) -> DependencyGraph {
 // Analizar imports de cada file
 // Construir grafo de dependencies
 // Si A importa B, cuando B cambia, A must recompilese
}
```

## See also

- `overview.md` — Pipeline completo de compilation
- `pipeline.md` — Orchestration current (without incremental)
