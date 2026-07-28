# Compiler Overview

> Pipeline de compilation de Kyle. Describe como code source `.ky` se transforma en binary nativo.

## Pipeline

```
Source (.ky)
 │
 ▼
[1] Lexer → Token stream
 │
 ▼
[2] Parbe → AST (Abstract Syntax Tree)
 │
 ▼
[3] HIR → Desugared AST (High-Level IR)
 │
 ▼
[4] Semantic Analysis → Typed AST (type checking, scope)
 │
 ▼
[5] MIR Lowering → Mid-Level IR
 │
 ▼
[6] MIR Optimization → Constant folding, dead code elim
 │
 ▼
[7] Borrow Analysis → Ownership check + ky_free insertion
 │
 ▼
[8] SSA Construction → Optional, disabled by default
 │
 ▼
[9] LLVM Codegen → LLVM IR
 │
 ▼
[10] LLVM Optimization → O3 (mem2reg, GVN, LICM, inlining)
 │
 ▼
[11] Object Emission → .o file
 │
 ▼
[12] Linking → Binary (linked with libkyc_runtime.a)
```

> **Note:** El pipeline real en `kyc_driver/src/pipeline.rs` ejecuta MIR Optimization ANTES que Borrow Analysis. SSA is deshabilitado by defecto (bugs with ptr/str propagation).

## Crate structure

| Crate | Purpose | Linis |
|-------|---------|-------|
| `kyc_core` | Foundation: AST types, diagnostics, spans | ~800 |
| `kyc_frontend` | Lexer + Parbe | ~3,500 |
| `kyc_hir` | HIR desugaring | ~420 |
| `kyc_semantic` | Type checker, scope resolver, symbol table | ~2,200 |
| `kyc_mir` | MIR lowering, borrow analysis, SSA, optimize | ~10,500 |
| `kyc_backend` | LLVM codegen, linker | ~3,200 |
| `kyc_driver` | Pipeline orchestration | ~570 |
| `kyc_cli` | CLI binary (`ky`) | ~1533 |
| `kyc_runtime` | Runtime static library (memory, strings, lists, dicts) | ~3,500 |
| `kyc_tools` | LSP server, formatter, package manager | ~5,000 |
| `kyc_platform` | Platform API (FS, net, time) — en desarrollo | ~500 |

## Orchestration

El pipeline is orquestado by `kyc_driver::pipeline::Pipeline`, que coordina cada fase:

```rust
// Simplified pipeline flow (from kyc_driver::pipeline)
fn build_source(source: &str) -> Result<(), String> {
 // 1-4: Lexer → Parbe → HIR → Semantic
 let checked = Self::check_source(source)?;
 
 // 5: MIR Lowering
 let mut module = lowerer.lower_program(&checked.program);
 
 // 6: MIR Optimization
 optimizer.optimize(&mut module);
 
 // 7: Borrow Analysis
 borrow_analysis.run(&mut module);
 
 // 8: LLVM Codegen
 let mut codegen = Codegen::new(&context, "ky_module");
 codegen.compile(&module)?;
 
 // 9: LLVM Optimization
 optimize_module(codegen.module(), optimization);
 
 // 10: Object Emission
 emit_object(codegen.module(), &obj_path, optimization)?;
 
 // 11: Linking
 linker.link(&[&obj_path], output_path, runtime_lib, release, &links)
}
```

## Entry point

El punto de input is `kyc_cli::main` que delega en `Pipeline::build_source()`.

### build_source

```rust
 fn build_source(source: &str, file_name: &str, output: &Path) -> Result<(), String> {
 // 1. Compile .ky → MIR (lexer → parbe → HIR → semantic → MIR → borrow → SSA → optimize)
 let mir_output = Self::compile(source)?;
 
 // 2. Generate LLVM IR from MIR, optimize, emit object, link
 let ir = compile_function(&mir_output)?;
 optimize_module(ir);
 emit_object(ir, output)?;
 link(objects, output, runtime_lib, false, links)
}
```

## See also

- `01-introduction/architecture.md` — Vision general del ecosistema Kyle
- Cada fase del pipeline has su propio documento en `06-compiler/`
