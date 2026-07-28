# Self-hosting — Portear Kyle de Rust a Kyle

## ¿Qué significa "Kyle corre Kyle"?

Hoy `ky` (el compilador) está escrito en **Rust** (10+ crates, ~50k líneas).
`libkyc_runtime.a` (el runtime) está escrito en **Rust** (~2k líneas).

**Kyle se compila a sí mismo** cuando el compilador `ky` está escrito en Kyle, y compila su propio código fuente.

---

## ¿Qué se puede pasar a Kyle AHORA?

Sin cambiar nada del compilador, usando solo `extern fn` + `@link` (FFI a C) + lógica pura:

### ✅ Runtime (YA se puede)

Cada función del runtime Rust se reescribe en Kyle. Las syscalls van por FFI.

```kyle
# Antes (Rust):
// kyc_runtime/src/memory.rs
pub extern "C" fn ky_alloc(size: i64) -> *mut u8 {
    unsafe { libc::malloc(size as usize) as *mut u8 }
}

# Después (Kyle):
# runtime/memory.ky
@link "c"
extern fn malloc(size: i64) ptr
extern fn free(ptr)

fn ky_alloc(size: i64) ptr:
    return malloc(size)

fn ky_free(p: ptr):
    free(p)
```

Es **exactamente lo mismo** que los packages (`http`, `json`, `sqlite`) — 100% Kyle con FFI a C.

| Componente del runtime | Líneas (Rust) | Portar a Kyle |
|------------------------|:------------:|:-------------:|
| `memory.rs` (alloc/free) | 80 | ✅ Ahora |
| `string.rs` (str ops) | 200 | ✅ Ahora |
| `list.rs` (List[T]) | 250 | ✅ Ahora |
| `dict.rs` (Dict[K,V]) | 180 | ✅ Ahora |
| `io.rs` (print, read) | 150 | ✅ Ahora |
| `net.rs` (TCP, WS) | 400 | ✅ Ahora (FFI a libc) |
| `channel.rs` (Chan[T]) | 100 | ✅ Ahora (FFI a pthread) |
| `thread.rs` (spawn) | 80 | ✅ Ahora (FFI a pthread) |
| `decimal.rs`, `datetime.rs`, `uuid.rs` | 300 | ✅ Ahora |
| `json.rs` | 200 | ✅ Ahora |
| **Total** | **~2,000** | **✅ Ahora (~2-3 semanas)** |

### ✅ Build System / CLI (YA se puede)

`ky build`, `ky run`, `ky check` son solo lógica + subprocesos:

```kyle
# Antes (Rust):
fn cmd_build(args: &[String]) {
    let source = std::fs::read_to_string(&file).unwrap();
    let mir = Pipeline::mir_source(&source, &file).unwrap();
    let codegen = Codegen::new(&context, "module");
    codegen.compile_with_ssa(&mir.module).unwrap();
    linker.link(&[&obj], output, ...).unwrap();
}

# Después (Kyle):
# cli/build.ky
extern fn system(cmd: &str) i32
extern fn read_file(path: &str) str
extern fn write_file(path: &str, content: &[u8])

fn cmd_build(source: str, file: str, output: str):
    # El parser, type checker, MIR, LLVM codegen
    # se llaman vía FFI a librerías Kyle compiladas
    ast = parse(source)
    hir = lower(ast)
    sem = check_types(hir)
    mir = lower_to_mir(sem)
    llvm_ir = codegen(mir)
    write_file(output + ".ll", llvm_ir)
    system("llc " + output + ".ll -o " + output + ".o")
    system("clang " + output + ".o -o " + output)
```

**Nota:** Todo EXCEPTO el codegen LLVM es lógica pura (strings, listas, match, if/for). Eso funciona HOY.

### ✅ Parser + Type Checker + MIR (YA se puede)

| Componente | Es lógica pura? | Se puede portar HOY? |
|-----------|:--------------:|:--------------------:|
| Lexer (caracteres → tokens) | ✅ Sí | ✅ Ahora |
| Parser (tokens → AST) | ✅ Sí | ✅ Ahora |
| HIR (AST desugar) | ✅ Sí | ✅ Ahora |
| Semantic (tipos, scopes) | ✅ Sí | ✅ Ahora |
| MIR (lower + optimize) | ✅ Sí | ✅ Ahora |
| SSA (mem2reg, phi) | ✅ Sí | ✅ Ahora |

Todos estos son algoritmos puros. No necesitan nada especial de Kyle. Strings, enums, pattern matching, listas, if/for — Kyle tiene todo eso.

---

## ¿Qué NECESITA una feature nueva de Kyle?

### 🟡 LLVM Codegen (requiere FFI bindings a LLVM C API)

El codegen traduce MIR → LLVM IR. Hoy usa `inkwell` (Rust bindings a LLVM).

Para portarlo a Kyle, necesitamos `extern fn` para la **C API de LLVM**:

```kyle
# llvm.ky — Bindings a LLVM C API
@link "llvm"
extern fn LLVMModuleCreateWithName(name: &str) ptr
extern fn LLVMAddFunction(mod: ptr, name: &str, ty: ptr) ptr
extern fn LLVMAppendBasicBlock(fn_val: ptr, name: &str) ptr
extern fn LLVMBuildRetVoid(builder: ptr)
extern fn LLVMBuildAdd(builder: ptr, lhs: ptr, rhs: ptr, name: &str) ptr
# ... ~200 funciones más

fn compile_module(mir: MirModule) -> ptr:
    mod = LLVMModuleCreateWithName("module")
    for func in mir.functions:
        fn_ty = make_fn_type(func.params, func.return_type)
        fn_val = LLVMAddFunction(mod, func.name, fn_ty)
        bb = LLVMAppendBasicBlock(fn_val, "entry")
        builder = LLVMCreateBuilder()
        LLVMPositionBuilderAtEnd(builder, bb)
        for inst in func.body:
            compile_inst(builder, inst)
    return mod
```

**La C API de LLVM existe y es estable.** Es el mismo camino que `http` con `libcurl`, `sqlite` con `libsqlite3`. Solo que ~200 funciones en vez de ~20.

### ❌ Inline Assembly

`asm("cli")` — necesario para el kernel, NO para el compilador. No bloquea self-hosting.

---

## ¿Qué se DEJA en Rust (por ahora)?

| Componente | Razón | Se puede pasar después? |
|-----------|-------|:----------------------:|
| **LLVM codegen** | ~200 funciones FFI a LLVM C API | ✅ Sí, cuando tengamos los bindings |
| **Optimizaciones** (LTO, inlining) | LLVM las maneja | ✅ Sí, vía bindings |
| **Linker** (clang/ld) | Solo llama a subprocesos | ✅ Sí, con `system()` FFI |

---

## Mapa de migración

```
HOY:  Kyle .ky ─▶ Rust parser ─▶ Rust types ─▶ Rust MIR ─▶ Rust LLVM ─▶ binary
                     (kyc_frontend)  (kyc_semantic)  (kyc_mir)  (kyc_backend)
                     + Rust runtime (kyc_runtime)

FASE 1 (2-3 sem):  Kyle .ky ─▶ Kyle parser ─▶ Kyle types ─▶ Kyle MIR ─▶ Rust LLVM ─▶ binary
                                  ↑               ↑             ↑
                              Portar a Kyle   Portar a Kyle  Portar a Kyle
                                  │               │             │
                                  └── Todo esto corre HOY en Kyle ──┘

                     Kyle runtime (portado de Rust)
                     ↑── Esto también corre HOY en Kyle

FASE 2 (4-6 sem):  Kyle .ky ─▶ Kyle parser ─▶ ... ─▶ Kyle MIR ─▶ Kyle LLVM bindings ─▶ binary
                                                                     (FFI a libLLVM)
                                                  Kyle runtime

FASE 3 (bootstrap): El Kyle viejo (Rust) compila el nuevo Kyle (Kyle)
                     El nuevo Kyle (Kyle) se compila a sí mismo
                     🎉 KYLE CORRE KYLE
```

## ¿Esto bloquea KYOS?

**NO.** KYOS puede construirse con el compilador Rust perfectamente. El self-hosting es un objetivo a largo plazo que no bloquea absolutamente nada de KYOS.

El flujo sería:

```
PARALELO:
  ├── Workstream A (KYOS Services) → Kyle user-space → compila con Rust ky
  ├── Workstream B (KYOS Desktop)  → KYUI nativo    → compila con Rust ky
  ├── Workstream C (KYOS Kernel)   → kernel Kyle     → compila con Rust ky + freestanding
  └── Workstream D (Self-hosting)  → portear Rust a Kyle → compilador Kyle en Kyle
```

Workstream D corre en segundo plano y no bloquea a nadie.

---

## Referencia de sintaxis

Para portar el compilador, usar `docs/15-kyle-syntax-reference.md` como referencia rápida de toda la sintaxis de Kyle con ejemplos funcionales.
