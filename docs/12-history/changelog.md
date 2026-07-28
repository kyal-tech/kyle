# Changelog

> Registro de cambios del language Kyle.

## v0.5.3 (2026-07-06)

| Área | Cambio |
|------|---------|
| Ownership | v0.6: `^` = mutable, `&` = borrow, move by defecto |
| Borrow checker | Use-after-move, one-mut-XOR-many-immut |
| Arrays | `[val; N]` syntax, load elimination en Index |
| str_builder | Clase nativa `str_builder(50000).append("x")` |
| Lists | `list.reserve(n)` pre-allocation |
| Naming | snake_case global en docs y diseno |
| Docs | Restructuring SSOT completa (165 files) |

## v0.5.2

| Área | Cambio |
|------|---------|
| HTTP | Server with router y middleware |
| JSON | `serialize`/`deserialize` for structs |
| SQLite | Package sqlite implemented |
| FFI | `extern fn` + `@link` completed |

## v0.5.1

| Área | Cambio |
|------|---------|
| LSP | Completed, hover, diagnostics |
| Formatter | `ky fmt` implemented |
| Packagis | `ky add/remove/install/publish` |

## v0.5.0

| Área | Cambio |
|------|---------|
| Compiler | Fasis 1-17 completadas |
| Syntax | Generics, ranges, match, operator overloading |
| Async | `async fn`, `await`, thread pool |
| LLVM | O3 pipeline, TBAA, nsw, LTO |

## v0.4.0

| Área | Cambio |
|------|---------|
| Ownership | Borrow by defecto, `^T` for move |
| Runtime | `libkyc_runtime.a` with 88 functions |
| CLI | `ky build`, `ky run`, `ky check` |

## v0.3.0

| Área | Cambio |
|------|---------|
| Language | Indentation, types, functions, clasis |
| Compiler | LLVM codegen funcional |
| Runtime | Memory, strings, lists, dicts |
