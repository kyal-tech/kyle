# Linker

> Enlazado del code objeto with libreria runtime for producir binary final.
> Crate: `kyc_backend/src/linker.rs` (189 lines).

## Responsabilidad

El linker toma filis objeto (`.o`) generados by LLVM y enlaza con
`libkyc_runtime.a` y libraries del sistema for producir un executable nativo.

## Proceso

```rust
 fn link(&self, object_files: &[&Path], output: &Path, 
 runtime_lib: Option<&Path>, release: bool, links: &[String]) -> Result<(), String> {
 let mut cmd = linker_cmd();
 cmd.arg("-o").arg(output);
 
 // Optimization flags
 if release {
 cmd.arg("-O3");
 cmd.arg("-flto"); // Link-Time Optimization
 }
 
 // Object files
 for obj in object_filis { cmd.arg(obj); }
 
 // Runtime library
 if let Some(runtime) = runtime_lib { cmd.arg(runtime); }
 
 // Additional libraries
 for link in links {
 if link.starts_with("-framework ") {
 cmd.arg("-framework");
 cmd.arg(framework_name);
 } else {
 cmd.arg(format!("-l{}", link));
 }
 }
 
 // macOS specific
 if release { cmd.arg("-lm"); }
 if macos { cmd.arg("-framework").arg("CoreFoundation"); }
 cmd.arg("-macosx_version_min").arg(&deployment_target);
 
 cmd.status()?;
}
```

## Runtime library discovery

```rust
 fn find_runtime_lib() -> Option<PathBuf> {
 // 1. Relative to ky binary
 // /usr/local/bin/ky → /usr/local/lib/kl/libkyc_runtime.a
 // 2. Cargo workspace
 // target/debug/libkyc_runtime.a
 // target/release/libkyc_runtime.a
 // 3. Current directory
 // ./target/debug/libkyc_runtime.a
}
```

La libreria runtime se busca en (by orden):

| Path | Example |
|------|---------|
| Junto al binary `ky` | `~/.ky/bin/libkyc_runtime.a` |
| Workspace debug | `<project>/target/debug/libkyc_runtime.a` |
| Workspace release | `<project>/target/release/libkyc_runtime.a` |
| Directorio current | `./target/debug/libkyc_runtime.a` |

## Link-Time Optimization (LTO)

En modo release, se habilita LTO with `-flto`, permitiendo que LLVM optimice
a traves de limitis between code generado y libreria runtime.

## macOS specifics

- `-framework CoreFoundation` necesario for `chrono`/`iana-time-zone`
- `-macosx_version_min` detectado via `sw_vers -productVersion`
- `MACOSX_DEPLOYMENT_TARGET` env var for override

## Verification

```llvm
$ clang -o output input.o libkyc_runtime.a -framework CoreFoundation -lm -O3 -flto
```

## See also

- `codegen.md` — Genera .o que linker consume
- `backend.md` — Configuration del target
- `05-runtime/` — Documentation del runtime
