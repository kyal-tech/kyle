# Kyle — Estado frente a Desarrollo de Kernels Nativos

**Versión:** 1.0
**Fecha:** 2026-08-01
**Objetivo:** Inventario completo de lo que Kyle **ya tiene implementado**, lo que está **a medias** y lo que **falta por completo**, con prioridad en que Kyle pueda desarrollar **kernels nativos, hipervisores, drivers, firmware y software de sistemas**.

> Leyenda: ✅ Implementado · 🟡 Parcial / a medias · ❌ No implementado

---

## 1. Estado General

Kyle compila a código máquina nativo vía **LLVM** (AOT, sin JIT), sin Garbage Collector y con runtime libremente enlazable. La base del lenguaje es sólida, pero para alcanzar un desarrollo real de kernels faltan piezas fundamentales: **inline assembly, memoria física/MMIO, interrupciones, no_std formal y varios tipos primitivos**.

La columna vertebral que YA funciona:
- Frontend (lexer, parser, HIR), type checker, borrow checker básico, MIR, codegen LLVM.
- Módulos, paquetes, FFI con C, funciones `extern`.
- Runtime rico: 190+ funciones `extern "C"`.
- Target `freestanding` que compila sin runtime (punto de partida para kernels).
- Target `wasm32` funcional.

---

## 2. Prioridad para Desarrollo de Kernels (top-down)

Orden recomendado de implementación para llegar a un kernel real:

| Prioridad | Feature | Estado | Notas |
|-----------|---------|--------|-------|
| 1 | Inline Assembly (`asm!`) | ❌ | Imprescindible: registros, barreras, interrupts |
| 2 | Punteros crudos + `volatile` | 🟡 | Existe `ptr`, falta `volatile read/write` para MMIO |
| 3 | Interrupciones (enable/disable/IRQ) | ❌ | Falta por completo |
| 4 | MMIO / DMA / Physical Memory | ❌ | Falta por completo |
| 5 | `no_std` + `no_allocator` formal | 🟡 | Target freestanding existe, falta formalizar |
| 6 | Tipos `i128/u128/f16/f128` | ❌ | — |
| 7 | ABI estable y exportar librerías | ❌ | — |
| 8 | Soporte de targets (ARM64, RISC-V) | 🟡 | Solo triple genérico + host x86_64 + wasm32 |

---

## 3. Compilador y Backend

| Feature | Estado | Detalle |
|---------|--------|---------|
| Compilación AOT nativa | ✅ | `ky run`/`ky build` → binario nativo |
| Backend LLVM | ✅ | `kyc_backend` completo |
| Sin JIT | ✅ | Compila y ejecuta el binario |
| Optimización múltiples niveles | ✅ | debug/release, opt via TargetMachine |
| LTO (ThinLTO) | ✅ | En modo release |
| Inlining (MIR + LLVM) | ✅ | `inline_small_functions` |
| DCE, loop unrolling, vectorización | 🟡 | Vía LLVM; no control explícito desde Kyle |
| PGO | ❌ | No existe |
| SIMD explícito | ❌ | No hay intrinsics ni types vectoriales |
| Inline Assembly (`asm!`) | ❌ | No existe en lexer/parser/MIR/codegen |
| DWARF / debug info | ❌ | No se emite información de depuración |
| Cross-compilation (ARM64, ARMv9, RISC-V) | 🟡 | Soporte por triple (`Codegen::new_with_target`, `<triple>-gcc`); sin targets oficiales |
| x86_64 host | ✅ | Target nativo actual |
| WebAssembly | ✅ | `wasm32` + `kyc_runtime_wasm` |

---

## 4. Sistema de Tipos

| Feature | Estado | Detalle |
|---------|--------|---------|
| Tipado estático + inferencia | ✅ | `:=` |
| `i8 i16 i32 i64 u8 u16 u32 u64 f32 f64` | ✅ | — |
| `i128`, `u128` | ❌ | No están en el type checker ni en `MirType` |
| `f16`, `f128` | ❌ | No existen |
| `bool`, `char`, `str` | ✅ | — |
| `byte` | 🟡 | No es tipo propio; se usa `u8` |
| `ptr` (raw pointer) | ✅ | `ptr`, cast `as ptr` |
| `*const` / `*mut` explícitos | 🟡 | Se modelan con `&` / `^` / `ptr`; sin sintaxis `*const/*mut` |
| `struct` / `class` | ✅ | Clases con campos/métodos |
| `enum` con payloads | ✅ | — |
| `union` | ❌ | No existe en AST |
| `tuple` | ✅ | `(T, U)` + acceso `.0` |
| Type Alias | ✅ | `Decl::TypeAlias` |
| NewType Pattern | ❌ | — |
| Phantom Types | ❌ | — |
| Const Generics | ❌ | — |
| Generics + monomorfización | ✅ | — |

---

## 5. Modelo de Memoria

| Feature | Estado | Detalle |
|---------|--------|---------|
| Ownership completo estilo Rust | 🟡 | Hay move/copy y borrow checker básico, pero no reglas formales de ownership |
| Borrowing `&` / `^` | ✅ | Referencias inmutables y mutables |
| Lifetimes (inferidos o explícitos) | ❌ | No existen |
| Move semantics | ✅ | `is_move_type` para tipos heap |
| Copy semantics | ✅ | Tipos valor |
| RAII / liberación automática | ❌ | Liberación manual (`ky_*_free`) |
| Stack allocation | ✅ | Allocas |
| Heap allocation | ✅ | `ky_alloc` |
| Placement allocation | ❌ | — |
| Arena allocators | ❌ | — |
| Custom allocators | ❌ | — |
| Control de alineación | ❌ | — |
| `unsafe` blocks | ✅ | Delimitados |
| mmap / memoria física | ❌ | No hay en runtime |
| MMIO | ❌ | — |
| DMA | ❌ | — |

---

## 6. Concurrencia

| Feature | Estado | Detalle |
|---------|--------|---------|
| Threads nativos | ✅ | `ky_spawn_thread` |
| Atomics | ✅ | `ky_atomic_*` (i64) |
| Mutex | ✅ | `ky_mutex_*` |
| Channels | ✅ | `ky_channel_*` + `chan<T>` |
| Async / Await | ✅ | `async fn`, `ky_spawn_task`, `ky_await_task`, `ky_yield`, `ky_parallel_for` |
| Executors | 🟡 | Task pool, no executor formal |
| Spinlock | ❌ | — |
| RWLock | ❌ | — |
| Semaphore | ❌ | — |
| Barrier | ❌ | — |
| Lock-free programming | ❌ | Solo atomics básicos |

---

## 7. Abstracciones de Alto Nivel

| Feature | Estado | Detalle |
|---------|--------|---------|
| Traits (como Rust) | ❌ | No existe `trait` en parser/AST |
| Smart / Unique / Shared / Weak pointers | ❌ | Solo raw `ptr`, `box<T>`, `chan<T>` |
| Function pointers | ✅ | `FnAddr` / callbacks |
| Macros (declarativas/procedurales/CT) | ❌ | — |
| CTFE / compile-time eval | ❌ | — |
| Reflection en compilación | ❌ | — |
| `const fn` / const evaluation | ❌ | — |
| Pattern matching con guards | ✅ | `MatchArm.guard` |
| Exhaustiveness checking | ❌ | No verificado |

---

## 8. Error Handling

| Feature | Estado | Detalle |
|---------|--------|---------|
| `Result` | ✅ | MIR + runtime |
| `Option` | ✅ | — |
| Errores custom | ✅ | — |
| Propagación `!` | ✅ | — |
| Sin excepciones | ✅ | — |

---

## 9. FFI y ABI

| Feature | Estado | Detalle |
|---------|--------|---------|
| FFI con C | ✅ | `@link "c"`, `extern fn` |
| FFI con C++ | ❌ | — |
| FFI con Rust | ❌ | Solo ABI C del runtime |
| Calling conventions: C | ✅ | Por defecto |
| Calling conventions: SysV | ❌ | — |
| Calling conventions: Windows | ❌ | — |
| Calling conventions: Fastcall | ❌ | — |
| Exportar librerías dinámicas (.so/.dll/.dylib) | ❌ | — |
| Exportar librerías estáticas (.a/.lib) | ❌ | — |
| ABI estable versionada | ❌ | — |
| Linker (clang/LLD) | ✅ | Integrado, con LLD en Windows |

---

## 10. Desarrollo de Kernels (Bare Metal)

| Feature | Estado | Detalle |
|---------|--------|---------|
| Target `freestanding` | ✅ | `ky build --target freestanding`, `ky new bare` |
| Compilar sin runtime | ✅ | `freestanding` omite el runtime |
| `no_std` / `no_runtime` | 🟡 | Vía target; no hay atributo formal en el lenguaje |
| `no_allocator` | 🟡 | Posible, no formalizado |
| Bootloader friendly | ❌ | — |
| MMIO acceso directo | ❌ | Falta `volatile` |
| DMA | ❌ | — |
| Interrupts | ❌ | — |
| Syscalls | ❌ | — |
| Scheduler friendly | 🟡 | Task pool a nivel de usuario, no de kernel |
| Hypervisor friendly | ❌ | — |

---

## 11. Seguridad

| Feature | Estado | Detalle |
|---------|--------|---------|
| Buffer overflow protection | ❌ | — |
| Stack protection | ❌ | — |
| Integer overflow detection | ❌ | — |
| Reducción de UB | 🟡 | Type checker + `unsafe` |
| Memory safety | 🟡 | — |
| Thread safety | 🟡 | — |
| Capability safety | ❌ | — |

---

## 12. Toolchain y Ecosistema

| Herramienta | Estado | Detalle |
|-------------|--------|---------|
| Compilador | ✅ | — |
| Linker | ✅ | clang/LLD |
| Formatter | ✅ | `ky fmt` |
| Linter | ❌ | — |
| Package Manager | ✅ | `ky add`, semver, lockfile, registry |
| Workspaces | ❌ | — |
| Build System integrado | ✅ | `ky build` |
| Test Runner | ✅ | `ky test` |
| Benchmark Runner | ❌ | — |
| Doc Generator | ❌ | — |
| LSP | ✅ | `ky lsp` |
| Debugger integration | ❌ | — |
| Cross Compiler | 🟡 | Triple genérico |

### Ecosistema de librerías

| Librería | Estado | Detalle |
|----------|--------|---------|
| Networking | ✅ | `net` runtime + paquete `http` + websocket |
| Cryptography | ✅ | sha256, base64, uuid |
| Serialization (JSON) | ✅ | `json` |
| TOML / YAML / XML | ❌ | — |
| Compression | ❌ | Solo doc |
| Async | ✅ | — |
| Collections | ✅ | list, dict, set, queue, stack, deque, linked_list |
| IO | ✅ | — |
| Filesystem | ✅ | `fs` |
| Math | ✅ | — |
| SIMD | ❌ | — |
| Time | ✅ | — |
| Unicode | 🟡 | `str` parcial; sin módulo completo |
| Database drivers | ✅ | `postgres`, `sqlite` |
| HTTP | ✅ | paquete `http` |
| TLS | ❌ | — |
| BigInt / Decimal | ✅ | runtime `big_int.rs`, `decimal.rs` |

### Módulos std pendientes

| Módulo | Estado | Detalle |
|--------|--------|---------|
| `std.cli` | ❌ | Solo documentado |
| `std.csv` | ❌ | Solo documentado |
| `std.bytes` | 🟡 | Wrapper listo; falta cerrar una línea de test residual |
| `std.log` | ✅ | Runtime `ky_log_*` + `log.ky` |

---

## 13. Resumen Ejecutivo

### Ya implementado (base sólida)
Lenguaje funcional completo (frontend, semantic, MIR, LLVM), FFI C, módulos, paquetes, testing, LSP, formatter, threads/async/atomics/channels/mutex, runtime de 190+ funciones, target `freestanding` y `wasm32`.

### A medias (prioridad de completar)
Cross-compilation real, `byte`, ownership formal, ejecutors, vectorización/SIMD, `no_std` formal, Unicode, `std.cli`, `std.csv`, `std.bytes`, workspaces.

### No implementado (lo que falta para kernels reales)
1. **Inline assembly** — imprescindible.
2. **Interrupciones, MMIO, DMA, memoria física, volatile** — imprescindible.
3. **`i128/u128/f16/f128`**.
4. **Traits**.
5. **Lifetimes / RAII / ownership formal**.
6. **Macros, CTFE, reflection, const fn**.
7. **union, NewType, Phantom, Const Generics**.
8. **Smart pointers**.
9. **Exportar librerías dinámicas/estáticas + ABI estable**.
10. **Debug info (DWARF)**.
11. **Seguridad: stack protector, overflow detection**.
12. **Doc generator, benchmark runner, linter**.
13. **PGO, SIMD**.

---

## 14. Siguientes pasos sugeridos

1. **Completar la suite de std modules y packages** (bytes, cli, csv, http, postgres, sqlite, env, crypto, config) — no requieren el compilador, solo runtime y wrappers. **Es lo que estabamos haciendo y podemos continuar de inmediato.**
2. Implementar **inline assembly** (lexer → parser → MIR → codegen LLVM).
3. Implementar **`volatile` + MMIO** (lectura/escritura sin optimización).
4. Añadir **`i128/u128/f16/f128`** al type checker y `MirType`.
5. Formalizar el **target ARM64/RISC-V** como triples oficiales.
6. Diseñar **Traits** (el cambio de lenguaje más grande pendiente).
