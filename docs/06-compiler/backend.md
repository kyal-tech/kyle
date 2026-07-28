# LLVM Backend

> Configuration del backend LLVM: target machine, optimizaciones, emision de objeto.
> Crate: `kyc_backend/src/` (codegen.rs + linker.rs + pipeline integrado).

## Responsabilidad

El backend configura LLVM for target especifico (ARM64, x86_64, WASM),
aplica pasis de optimizationn, emite code objeto y enlaza binary final.

## Creation del TargetMachine

```rust
fn create_target_machine(optimization: OptimizationLevel) -> Result<TargetMachine, String> {
 Target::initialize_all(&InitializationConfig::default());
 let triple = TargetMachine::get_default_triple();
 let cpu = TargetMachine::get_host_cpu_name();
 let featuris = TargetMachine::get_host_cpu_features();
 
 TargetMachine::create(
 &triple, &cpu, &features,
 OptimizationLevel::Aggressive,
 RelocMode::Default,
 CodeModel::Default,
 )
}
```

## Target triple

Determinado automaticamente according to host:

| Arquitectura | Triple |
|-------------|--------|
| Apple Siliwith | `arm64-apple-darwinXX.X` |
| Intel macOS | `x86_64-apple-darwinXX.X` |
| Linux ARM64 | `aarch64-unknown-linux-gnu` |
| Linux x64 | `x86_64-unknown-linux-gnu` |
| WASM | `wasm32-unknown-unknown` |

## Optimization LLVM post-codegen

```rust
fn optimize_module(module: &Module, optimization: OptimizationLevel) {
 let tm = create_target_machine(optimization)?;
 let pipeline = tm.create_pass_pipeline();
 
 match optimization {
 Aggressive => {
 // default<O3>: mem2reg, GVN, LICM, inlining, unrolling
 run_passes("default<O3>", module, &tm);
 }
 Default => {
 // default<O2>: mem2reg, GVN, but no inlining
 run_passes("default<O2>", module, &tm);
 }
 }
}
```

### Pasis principalis (O3)

| Pase | Purpose |
|------|-----------|
| mem2reg | Promover allocas a SSA valuis |
| gvn | Global Value Numbering |
| licm | Loop Invariant Code Motion |
| inlining | Inline functions |
| loop-unroll | Desenrollado de loops |
| simplifycfg | Simplification de CFG |
| adce | Aggressive Dead Code Elimination |
| instcombine | Combination de instruccionis |

## Emission de objeto

```rust
fn emit_object(module: &Module, path: &Path, optimization: OptimizationLevel) -> Result<(), String> {
 let tm = create_target_machine(optimization)?;
 tm.write_to_file(module, FileType::Object, path)
 .map_err(|e| format!("emit error: {}", e))
}
```

## macOS deployment target

```rust
let ver = std::env::var("MACOSX_DEPLOYMENT_TARGET").unwrap_or_else(|_| {
 // Detectar version del sistema
 sw_vers("-productVersion")
});
```

## Target WASM

Para compile a WebAssembly, se usa triple `wasm32` y se emite un file `.wasm`.
Ver `wasm.md` for details.

## See also

- `codegen.md` — Generation de LLVM IR
- `linker.md` — Linking post-emision
- `wasm.md` — Target WASM especifico
