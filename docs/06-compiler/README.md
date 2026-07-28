# 06-compiler

> Documentation del compiler de Kyle: pipeline completo, from code source hasta binary.

## Files

| Documento | Description | Status |
|-----------|-------------|--------|
| `overview.md` | Pipeline general y arquitectura del compiler | ✅ |
| `pipeline.md` | Orchestrador que conecta todas fasis | ✅ |
| `lexer.md` | Tokenization: source → tokens | ✅ |
| `parser.md` | Parsing: tokens → AST | ✅ |
| `ast.md` | Definicionis del Abstract Syntax Tree | ✅ |
| `hir.md` | High-Level IR: desugaring | ✅ |
| `semantic.md` | Type checking, scope resolution | ✅ |
| `mir.md` | Mid-Level IR: lowering | ✅ |
| `borrow-analysis.md` | Analysis de ownership y borrows | ✅ |
| `ssa.md` | Transformation a SSA form | ✅ |
| `optimizer.md` | Optimizacionis a nivel MIR | ✅ |
| `codegen.md` | Generation de LLVM IR | ✅ |
| `backend.md` | Configuration LLVM target | ✅ |
| `linker.md` | Linking with runtime library | ✅ |
| `diagnostics.md` | Sistema de errors y warnings | ✅ |
| `incremental.md` | Compilation incremental | 🔶 Designed, no implemented |
| `wasm.md` | Target WebAssembly | ✅ |
