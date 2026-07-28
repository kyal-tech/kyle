# Pipeline

> Orchestrador que conecta todas fasis del compiler.
> Crate: `kyc_driver/src/pipeline.rs` (566 lines).

## Responsabilidad

El pipeline coordina execution secuencial de todas fasis del compiler:
desde code source hasta binary final.

## Pipeline functions

### build_source

```rust
 fn build_source(source: &str, file_name: &str, output_path: &Path) -> Result<(), String> {
 // Dev builds: no LTO
 let mir_output = Self::compile(source)?;
 let module = compile_functions(&mir_output)?;
 optimize_module(&module, OptimizationLevel::Default);
 emit_object(&module, obj_path, OptimizationLevel::Default)?;
 linker.link(&[obj_path], output_path, runtime_lib, false, &links)
}
```

### build_source_release

```rust
 fn build_source_release(source: &str, file_name: &str, output_path: &Path) -> Result<(), String> {
 // Release builds: -O3 + LTO
 let mir_output = Self::compile(source)?;
 let module = compile_functions(&mir_output)?;
 optimize_module(&module, OptimizationLevel::Aggressive);
 emit_object(&module, obj_path, OptimizationLevel::Aggressive)?;
 linker.link(&[obj_path], output_path, runtime_lib, true, &links)
}
```

### compile

```rust
fn compile(source: &str) -> Result<MirOutput, String> {
 // 1. Lexer: source → tokens
 let tokens = lexer::tokenize(source)?;
 
 // 2. Parser: tokens → AST (with error recovery)
 let (module, parse_errors) = parser::parse(&tokens);
 if !parse_errors.is_empty() { return Err(format_errors(parse_errors)); }
 
 // 3. HIR: desugar AST
 let hir = hir::desugar(&module);
 
 // 4. Semantic: type check + scope
 let mut type_checker = TypeChecker::new();
 type_checker.check_module(&hir)?;
 let typed_ast = type_checker.typed_ast();
 
 // 5. MIR Lowering: AST → MIR
 let mut mir = mir::lower(typed_ast);
 
 // 6. Borrow Analysis
 let mut borrow_analysis = BorrowAnalysis::new();
 borrow_analysis.run(&mut mir);
 if !borrow_analysis.errors().is_empty() { return Err(join_errors(borrow_analysis.errors())); }
 
 // 7. SSA Construction
 ssa::transform(&mut mir);
 
 // 8. MIR Optimization
 optimize(&mut mir);
 
 Ok(MirOutput { module: mir, ... })
}
```

## CLI integration

```rust
// kyc_cli/src/main.rs
fn main() {
 let args = parse_args();
 let source = read_file(&args.file);
 
 let result = if args.release {
 Pipeline::build_source_release(&source, &args.file, &args.output)
 } else {
 Pipeline::build_source(&source, &args.file, &args.output)
 };
 
 match result {
 Ok(()) => println!("Build complete: {}", args.output),
 Err(e) => eprintln!("Build error: {}", e),
 }
}
```

## Modes

| Mode | Command | Optimization | LTO | Uso |
|------|---------|-------------|-----|-----|
| Debug | `ky build` | Default (-O0) | No | Desarrollo |
| Release | `ky build --release` | Aggressive (-O3) | Si | Production |
| Check | `ky check` | Only analysis | No | Type-check rapido |

## See also

- `overview.md` — Diagrama del pipeline
- `lexer.md` → `parser.md` → `hir.md` → `semantic.md` → `mir.md` → `borrow-analysis.md` → `ssa.md` → `optimizer.md` → `codegen.md` → `linker.md`
