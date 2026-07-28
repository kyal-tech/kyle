# Roadmap

## Platform Architecture

Kyle is a layered platform. Each layer only knows the layer below it.

```
Applications (IDE, Desktop, KYOS...)
 │
Kyle UI (widgets, navigation)
 │
Kyle Scene (scene graph, layout)
 │
Kyle Graphics (canvas, GPU)
 │
Kyle Windowing (windows, events)
 │
Kyle Platform (FS, net, threads, audio, sensors)
 │
Kyle Runtime (memory, strings, collections)
 │
Kyle Language (compiler)
```

The **compiler and runtime are complete** (Phasis 1-17). 
The **upper layers (Windowing → Applications) are aspirational — without fecha estimada**. 
El foco current are **packagis backend** (HTTP, SQLite, Postgres, websocket).

### Current crate structure

```
kyc_core/ ✅ foundation
kyc_frontend/ ✅ lexer + parser
kyc_hir/ ✅ desugaring
kyc_semantic/ ✅ type checker, scope
kyc_mir/ ✅ MIR lowering, SSA
kyc_backend/ ✅ LLVM codegen
kyc_driver/ ✅ pipeline
kyc_cli/ ✅ CLI binary
kyc_runtime/ ✅ runtime (memory, strings, lists, dicts)
kyc_tools/ ✅ LSP, formatter, package manager
```

### Current crate structure (updated)

| Crate | Status | Purpose |
|-------|--------|---------|
| `kyc_core` | ✅ | Foundation types |
| `kyc_frontend` | ✅ | Lexer + parser |
| `kyc_hir` | ✅ | Desugaring |
| `kyc_semantic` | ✅ | Type checker, scope, borrow |
| `kyc_mir` | ✅ | MIR lowering, SSA, optimizations |
| `kyc_backend` | ✅ | LLVM codegen, linker |
| `kyc_driver` | ✅ | Pipeline orchestration |
| `kyc_cli` | ✅ | CLI binary (`ky`) |
| `kyc_runtime` | ✅ | Runtime (memory, strings, lists, dicts) |
| `kyc_tools` | ✅ | LSP, formatter, package manager |
| `kyc_ui` | ✅ | .kyx parser + UI-IR + Backends |
| `kyc_platform` | ✅ | FS, Time APIs |
| `kyc_graphics` | 📅 | Aspirational |
| `kyc_scene` | 📅 | Aspirational |

---

## ✅ Completed — Phasis 1–17

| Phase | Description | Status |
|-------|-------------|--------|
| 1–2 | Documentation and specification | ✅ |
| 3 | Lexer | ✅ |
| 4 | Parbe | ✅ |
| 5 | HIR + desugaring | ✅ |
| 6 | Semantic analysis (type checker, scope) | ✅ |
| 7 | Borrow semantics (ownership, move, copy, borrow checker) | ✅ **v0.6** |
| 8 | Backend release mode (LLVM, binary output) | ✅ |
| 9 | Async scheduler V2 (thread pool) | ✅ |
| 10 | Iterators — 17 list methods (map, filter, fold, etc.) | ✅ |
| 11 | Package manager (registry, cache, lock, publish, login) | ✅ |
| 12 | Tooling (LSP, VS Code extension, formatter, test framework) | ✅ |
| 13 | Complete syntax (generics, ranges, match, operator overloading, is, ptr, for-else, static fn, **) | ✅ |
| 14 | Referencis and borrow checker (`^T` mutable, `&T` borrow, `^&T` mut borrow, field mutability) | ✅ **v0.6** |
| 15 | SSA form (mem2reg, phi nodes, GVN, dominator fix) | ✅ |
| 16 | LLVM IR quality (nsw, TBAA, inbounds, readonly, noalias, noundef, !range, align, lifetime) | ✅ |
| 17 | Optimization pipeline (O3, constant folding, alloca elimination, nsw verification) | ✅ |

**Benchmarks (Kyle vs Rust vs C) — currentizado Jul 2026:**

Ejecutado en Apple M4 (ARM64). Compiladores: `clang -O3`, `rustc -O`, `ky build`.

| Benchmark | Kyle | Rust | C | Relation (K/C) |
|-----------|:----:|:----:|:-:|:--------------:|
| Primis 3M | 672ms | 467ms | 444ms | 1.51x |
| Fibonacci 40 | 406ms | 423ms | 410ms | **0.99x** |
| Mandelbrot (float) | 294ms | 274ms | 271ms | 1.08x |
| String concat 100k | 1.98s | 272ms | 238ms | 8.3x |
| **Compile memory** | **7.0MB** | 18.9MB | 2.6MB | **2.7x vs Rust** |

Kyle compite directamente with C y Rust en CPU-bound (primes, fib, mandelbrot).
La brecha en strings se must a que Kyle usa `str_concat` (nueva alloc + copy en cada `+`).
Rust usa `String::push` que amplia buffer existente.

Kyle usa 2.7x less RAM que Rust during compilation (without LTO).

Ver `BENCHMARK.md` for suite completo y `docs/` for featuris faltantes.

---

## 🔜 Immediate Next — Phase 0: FFI Foundation

**Estimated: 2.5 days** 
**Crate restructuring: NOT required for Phase 0** — all changis are in existing compiler crates.

Goal: Enable packagis written in 100% Kyle without Rust.

| Step | Task | Filis | Est. |
|------|------|-------|------|
| 0.1 | `extern fn` declaration — parser, semantic, MIR, codegen | `parser.rs`, `type_checker.rs`, `lower.rs`, `codegen.rs` | 1 day |
| 0.2 | `@link` directive — parbe + linker integration | `parser.rs`, `pipeline.rs` | 0.5 day |
| 0.3 | `ptr` type complete — load, store, offset, arithmetic | `lower.rs`, `codegen.rs` | 1 day |

After Phase 0: packagis like `ky-http`, `ky-sqlite`, `ky-json` can be written in pure Kyle with FFI to C libraries.

---

## Runtime Rewrite Analysis — What Kyle Can and Cannot Do

The runtime (`libkyc_runtime.a`) has **88 `extern "C"` functions**. Here's exactly what can be rewritten in pure Kyle (+ `extern fn` to libc) and what cannot.

### ✅ Can rewrite NOW (65 functions — 74%)

These use only arithmetic, raw pointers, and libc FFI. All doable with current Kyle features.

| Module | Count | Examplis |
|--------|:-----:|----------|
| `string.rs` | 20 | `ky_strlen`, `ky_concat`, `ky_str_to_i64`, `ky_str_contains`, `ky_substr`, etc. |
| `list.rs` | 28 | `ky_list_new/push/get/set/pop/len`, `ky_list_map/filter/fold`, `ky_range`, etc. |
| `io.rs` | 10 | `ky_print/println`, `ky_open/read/write/close`, `ky_sleep/now` |
| `lib.rs` | 4 | `ky_pow`, `ky_add_pct`, `ky_sub_pct`, `ky_mul_pct` |
| `assert.rs` | 1 | `ky_assert` |
| `async_.rs` | 1 | `ky_yield` |

**Total: 65 functions. Estimated: 2-3 days.**

### 🔶 Needs missing Kyle feature (12 functions — 14%)

| Function | What's Missing | Workaround |
|----------|---------------|------------|
| `ky_alloc/ky_free` | **Heap allocator** | Add `extern fn malloc(size: i64) ptr` ✅ already possible |
| `ky_retain/ky_release` | **Atomic operations** | Needs LLVM `atomicrmw` instruction or `__sync_fetch_and_add` via extern |
| `ky_iter_new/next/map/filter/free` (5) | **`box::new` + function ptr transmute** | Use `extern fn malloc` for heap + manual vtable |
| `ky_f64_to_str` | **f64 formatting** | Use `extern fn snprintf(buf, size, fmt, val)` from libc |
| `ky_assert_eq/assert_ne` | **i64 error message formatting** | Use `ky_i64_to_str` (from ✅ group) |

**Estimated: 2-3 days (after adding missing extern fn declarations).**

### ❌ Cannot rewrite (12 functions — 14%)

| Module | Count | Why |
|--------|:-----:|-----|
| `dict.rs` | 10 | ✅ All 10 functions implemented (`ky_dict_contains`, `ky_dict_remove` added). Exposed as `dict.contains(d, key)` / `dict.remove(d, key)`. |
| `thread.rs` | 2 | **Needs OS threads** — `ky_spawn_thread` needs `pthread_create` with a C-compatible trampoline. `ky_join_thread` needs `pthread_join`. The trampoline requiris codegen support for `extern "C"` closures. |
| `async_.rs` | 2 | **Needs async executor** — `ky_spawn_task` and `ky_await_task` depend on threads, channels, mutexes, atomics, and a global executor singleton. This is the most complex piece. |

**Estimated: 2-3 weeks (hash map: 1 week, threads: 1-2 weeks, async: depends on threads).**

### Summary

| Status | Count | % |
|--------|:-----:|:-:|
| ✅ Can rewrite NOW | 65 | 74% |
| 🔶 Needs minor feature | 12 | 14% |
| ❌ Needs major feature | 12 | 14% |
| **Total** | **88** | **100%** |

---

## Runtime Rewrite Plan — ⏸️ PAUSADO

> **Decision:** La rewrite del runtime Rust a Kyle is pausada hasta que Kyle se use en proyectos reales. 
> El file `crates/kyc_runtime/src/string.ky` existe with 18 functions pero permanece inerte. 
> Reactivar cuando: (1) proyectos realis requieran auto-suficiencia, (2) equipo lo considere necesario.

| Phase | Description | Status |
|-------|-------------|--------|
| A | Rewrite 65 functions (string, list, io) | ⏸️ Pausado |
| B | Missing extern fn declarations | ⏸️ Pausado |
| C | Hash map (Dict) en Kyle puro | ⏸️ Pausado |
| D | Threading & Async | ⏸️ Pausado |
| E | Self-hosting compiler | 📅 Baja prioridad |

---

## Implementation Order

```
AHORA → Backend Packagis + websocket/WASM — ✅ ENFOQUE ACTUAL
 ↓
 Runtime rewrite en Kyle — ⏸️ PAUSADA (hasta tener proyectos reales)
 ↓
 Windowing, Graphics, Scene, UI — 📅 ASPIRACIONAL (without fecha)
 ↓
 KYOS (sistema operativo) — 📅 ASPIRACIONAL (without fecha)
 ↓
 Self-hosting — 📅 BAJA PRIORIDAD
```

### Fasis HTTP/JSON

| Fase | Description | Depende de | Status |
|------|-------------|------------|--------|
| 1 | Function pointers (`fn()` as type de primera clase) | Compiler | ✅ |
| 2 | `JsonValue` type + auto-serialize de `final class` | Union typis | ✅ |
| 3 | HTTP client: `client.post(url, class)` auto-JSON | Fase 2 | ✅ |
| 4 | HTTP Server: callbacks, `{id:i32}` params, middleware | Fase 1 + 3 | 🔜 |
| 5 | websocket + SSE about Server | Fase 4 | 🔜 |

**Current state:** Packagis work (http client, json, sqlite, env) in 100% Kyle with FFI. HTTP Server TCP accept working. Function pointers implemented. Benchmarks completeds (Kyle vs Rust vs C). Bugs recientis arreglados: constructor without `fn`, `@link` propagation, `use X.{a, b, c}`, tuple destructuring, `close()` name conflict.

**Runtime rewrite en Kyle: PAUSADA.** El file `crates/kyc_runtime/src/string.ky` existe with 18 functions pero NO se usa. Reactivar solo cuando Kyle se use en proyectos realis y auto-suficiencia sea necesaria. Mientras tanto, runtime Rust funciona y is estable.

### Bugs conocidos (no criticos)

| Bug | Impacto | Workaround |
|-----|---------|------------|
| Closuris as argumentos truncados a i32 | Bajo | Usar functions nombradas |

## Package Registry

Kyle packagis are distributed via **GitHub Pagis registry** at `https://IT-KYNERA.github.io/KYLE/docs`. No `KL_REGISTRY` needed.

### Available packages

| Package | Registry | Status |
|---------|----------|--------|
| `http` | GitHub Pagis | ✅ v3.0 (cliente) |
| `json` | GitHub Pagis | ✅ v0.1.0 |
| `sqlite` | GitHub Pagis | ✅ v0.1.0 |
| `env` | GitHub Pagis | ✅ v0.1.0 |

### Development

```bash
# Local — apuntar al file registry del repo
export KL_REGISTRY=file:///Users/me/KYLE/registry
ky add http
```

See `docs/05-packages/registry.md` for full documentation.

---

## Status real del compiler (verificado 2026-07-04)

### ✅ Todo funciona — without bugs conocidos

| Feature | Example |
|---------|---------|
| Functions importadas de modulis | `use http.server.send_json` ✅ |
| Clasis importadas de modulis | `use http.server.router` ✅ |
| Struct return with strings | `fn json_res(data: str) Res: Ris { body: data }` ✅ |
| Generic function calls with `<T>` | `identity<i32>(42)`, `deserialize<LoginReq>(body)` ✅ |
| Generic function/class decls with `<T>` | `fn identity<T>(x: T) T:`, `class box<T>:` ✅ |
| Type alias with `<T>` | `type Maybe<T> = T?` ✅ |
| `serialize(val)` / `deserialize<T>(str)` | Builtin global ✅ |
| Clasis + methods | `client.get(url)` ✅ |
| Struct literal with type args | `Container<i32> { value: 42 }` ✅ |
| Multi-level module imports | `use http.server....` ✅ |
| Package @link propagation | `@link "c"` se mergea al program automaticamente ✅ |
| **Implicit main** | `ky run app.ky` without `fn main()` — code module ejecutado automaticamente ✅ |
| **Closuris multi-line** | `(req, client):\n send_json(...)` with bloque indentado ✅ |
| **Function pointers** | `handler as ptr`, llamado via `fn_ptr(args)` with CallIndirect ✅ |
| **router HTTP alto nivel** | `app.get("/path", handler)` with `get` as metodo (keyword) ✅ |
| **Packagis without std/** | `use http.server....` busca directamente en `packages/http/src/` ✅ |

### 🔜 En desarrollo

| Feature | Example | Status |
|---------|---------|--------|
| `res.json()` | `res.json(serialize(data), 200)` | ✅ Implementado |
| `req.param()` | `req.param("id")` | ✅ Implementado |
| `req.body<T>()` | `req.body<CreateUser>()` | Requiere generic method call |
| `obj.method<T>(args)` | Llamada a metodo with type generico | Parbe + lowerer |

---

## Typis faltbefore — Plan de implementation

> La specification completa de cada type is en [`docs/03-language-reference/types.md`](docs/03-language-reference/types.md) section "Platform typis (futuro)".
> Todos nombris de type van en **minuscula**: `datetime`, `bytes`, `decimal`, `uuid`, `url`, `regex`, `duration`.

### Typis currentis de Kyle

| Category | Typis |
|-----------|-------|
| **Primitivos** | `i8`, `i16`, `i32`, `i64`, `f32`, `f64`, `bool`, `char`, `str` |
| **Compuestos** | `{T}` (list), `{K: V}` (dict), `[T, N]` (array), `ptr` |
| **Functions** | `fn(T) U` (function pointers) |
| **Usuario** | `class`, `struct`, `enum`, `type alias` |
| **Generics builtin** | `Option<T>` (`T?`), `Result<T, str>` (`T!`) |

### Typis faltbefore with prioridad alta (necesarios for PostgreSQL)

| Type | Description | Subtypes/Operacionis | Depende de |
|------|-------------|---------------------|------------|
| `datetime` | Fecha y hora | `date`, `time` — sumar/rbe days/horas, formatear, parsear | Runtime en Rust |
| `bytes` | Datos binarys | `len()`, `[i]`, `slice()`, `hex()`, `base64()` | Runtime en Rust |
| `decimal` | Precision fija | `round()`, `truncate()`, operacionis aritmeticas | Runtime en Rust |

### Typis faltbefore with prioridad media

| Type | Description | Operacionis |
|------|-------------|-------------|
| `uuid` | Identificador unico | `uuid.v4()`, `uuid.parse()`, `uuid.to_string()` |
| `url` | Localizador | `url.scheme()`, `url.host()`, `url.path()` |
| `regex` | Expression regular | `regex.match()`, `regex.find()`, `regex.replace()` |
| `duration` | Intervalo de tiempo | `duration.seconds()`, `duration.hours()`, `a + b` |

### 🏷️ Convention de nombris (camelCase)

| Regla | Examplis |
|-------|----------|
| Typis 1 palabra | `regex`, `url`, `uuid`, `bytes`, `decimal`, `mutex`, `future`, `box` |
| Typis multi-palabra | `str_builder`, `atomic_i64`, `atomic_bool`, `big_int`, `date_time` |
| Functions | `spawn_thread`, `join_thread`, `parallel_for`, `fetch_add`, `to_str` |
| Constructoris type | `regex("[0-9]+")`, `box(42)`, `channel<i32>(16)` |
| Constbefore | `MAX_SIZE := 1024` (UPPER) |

### Nombris a renombrar (code existente)

| Actual | Nuevo | Filis |
|--------|-------|----------|
| `ky_parallel_for` | `ky_parallel_for` (futuro: `parallel.for`) | runtime, symbol_table, lowerer |
| `ky_spawn_thread` | `ky_spawn_thread` (futuro: `thread.spawn`) | runtime, symbol_table |
| `ky_join_thread` | `ky_join_thread` (futuro: `thread.join`) | runtime, symbol_table |
| `ky_str_builder_*` | `str_builder` clase | ✅ ya wrapper |
| Functions package | camelCase | packages/http/, packages/json/ |

### Decision: Stack y Queue no are typis dedicados

| Type | Decision | Razon |
|------|----------|-------|
| **Stack** | ❌ No there is type dedicado | `{T}` with `push()`/`pop()` ya is stack. Go y JS hacen lo mismo. |
| **Queue** | ❌ No there is type dedicado | `{T}` with `push()`/`pop_first()` cubre FIFO. |
| **set** | ✅ Si, type dedicado | Requiere hash set (without duplicados, O(1) lookup). |

## ⚠️ Option<T> y Result<T,E> — What does it mean?

`Option<T>` (`T?`) y `Result<T, str>` (`T!`) existen as **enums genericos** del compiler:

```kyle
x = maybe_value? # T? = Option<T>
y = fallible_operation! # T! = Result<T, str>

if x is Some(val): # pattern matching en Option
 print(val)
```

Estan implementeds with `Option.Some(value)` / `Option.None` y `Result.Ok(value)` / `Result.Error(msg)`. La syntax `T?` y `T!` is azucar del compiler.

**Limitations:** No have methods ricos as `map()`, `and_then()`, `unwrap_or()` (tipicos de Rust). Solo pattern matching basico y operador `??` (default).

### Plan de fases

| Fase | Typis | Filis | Esfuerzo |
|------|-------|----------|----------|
| **1** | `datetime` + `duration` (runtime) | `kyc_runtime/src/datetime.rs` + FFI | 3-4 days |
| **2** | `bytes` (runtime) | `kyc_runtime/src/bytes.rs` + FFI | 2-3 days |
| **3** | `decimal` (runtime) | `kyc_runtime/src/decimal.rs` + FFI | 2-3 days |
| **4** | `uuid` (runtime) | `kyc_runtime/src/uuid.rs` + FFI | 1-2 days |
| **5** | Methods ricos en `Option<T>`/`Result<T,E>` | Compiler + runtime | 2-3 days |

Cada type nuevo necesita:
1. **Struct en runtime** (`kyc_runtime/src/`) with operacionis basicas
2. **Extern functions** (`@link "c"` + `extern fn`)
3. **Package Kyle** (`packages/<type>/`) with API de alto nivel
4. **Tests** en package

---

## Self-Hosting — Codegen Analysis

To compile Kyle with Kyle, the compiler's codegen (`kyc_backend/src/codegen.rs`, ~2,400 linis of Rust) must be rewritten in Kyle. It currently usis **inkwell** (Rust wrapper for LLVM C API).

### What would be needed

| Component | Linis | Difficulty |
|-----------|:-----:|:----------:|
| LLVM C API `extern fn` declarations | ~85 functions | 🟢 Easy (one-time typing) |
| LLVM type wrappers (`LLVMValueRef`, etc.) | ~100 | 🟢 Easy (all `ptr`) |
| Codegen logic (translate MIR → LLVM IR) | ~2,400 | 🔴 Hard (complex dispatch logic) |
| SSA construction | ~920 | 🟡 Medium (already pure algorithms) |
| Linker driver | ~150 | 🟢 Easy (`system("clang ...")` already works) |

### LLVM is NOT replaced

LLVM stays as the machine-code backend. The same way we call libcurl via `extern fn`, we would call LLVM via `extern fn LLVMBuildAdd(...)`. The ~85 LLVM C API functions map 1:1 to the existing inkwell calls.

**Verdict:** Technically feasible. Not a priority — the Rust compiler is stable and the codegen doesn't need to be self-hosted to ship packages.

### Plan de auto-hospedaje (Self-Hosting)

```kyle
# El objetivo: ky compilado by ky
ky build compiler.ky # current: Rust → binary
 # futuro: Kyle → binary
```

#### Fase 1 — LLVM C API bindings (`packages/llvm`)

Declarar ~85 functions de LLVM C API as `extern fn` en Kyle:

```kyle
@link "LLVM"
extern fn LLVMContextCreate() ptr
extern fn LLVMModuleCreateWithName(name: ptr) ptr
extern fn LLVMAddFunction(mod: ptr, name: ptr, type_: ptr) ptr
# ... ~82 mas
```

| File | Functions | Esfuerzo |
|---------|-----------|----------|
| `packages/llvm/src/core.ky` | Context, Module, Type, Value | 40 functions |
| `packages/llvm/src/builder.ky` | IRBuilder (BuildAdd, BuildCall...) | 30 functions |
| `packages/llvm/src/target.ky` | TargetMachine, emit object | 15 functions |

#### Fase 2 — MIR typis en Kyle (`packages/mir`)

Los typis del compiler (hoy en Rust) se define en Kyle:

```kyle
final class MirType:
 kind: i32 # 0=i32, 1=str, 2=struct...
 name: str

final class MirInst:
 opcode: i32
 dest: i32
 args: &[i32]
```

| File | Contenido |
|---------|-----------|
| `packages/mir/src/types.ky` | MirType, MirValue, MirConstant |
| `packages/mir/src/inst.ky` | MirInst (25 variants) |
| `packages/mir/src/function.ky` | MirFunction, MirBasicBlock |

#### Fase 3 — Codegen en Kyle (`packages/codegen`)

Traducir `codegen.rs` (~2,400 lines Rust → Kyle):

```kyle
use llvm.{core, builder}
use mir.{types, inst, function}

fn compile_function(f: MirFunction) ptr:
 fn_type = llvm_fn_type(f.return_type, f.params)
 llvm_fn = LLVMAddFunction(module, f.name, fn_type)
 # ...
```

| Module | Lines | Dificultad |
|--------|:------:|:----------:|
| Type lowering | ~300 | 🟢 |
| Function compilation | ~800 | 🟡 |
| Instruction compilation | ~1,000 | 🔴 |
| SSA construction | ~300 | 🟡 |

#### Fase 4 — Bootstrap

```bash
# 1. Compilar codegen Kyle usando compiler Rust
ky build codegen.ky -o ky-step1

# 2. Compilar codegen Kyle usando ky-step1
./ky-step1 codegen.ky -o ky-step2

# 3. Verificar que ky-step2 produce mismo output
diff <(ky-step1) <(ky-step2) # bootstrap completed!
```

#### Timeline estimado

| Fase | Description | Dependencias | Esfuerzo |
|------|-------------|-------------|----------|
| 1 | LLVM C API bindings | — | 1 day |
| 2 | MIR typis en Kyle | — | 2 days |
| 3 | Codegen en Kyle | Fase 1 + 2 | 2 semanas |
| 4 | Bootstrap | Fase 3 | 1 day |
| 5 | Rewrite frontend (lexer+parser) en Kyle | Fase 4 | 4 semanas |

---

## 📅 Post-v1.0 Features

### Phase 18 — Zero-Cost Abstractions

| # | Task | Priority |
|---|------|----------|
| 18.1 | Escape analysis: `final class` on stack instead of heap | ⭐⭐⭐⭐ |
| 18.2 | Small string optimization (SSO): strings < 15 bytis inline | ⭐⭐⭐ |
| 18.3 | Inlining `.map()`/`.filter()`/`.fold()` — zero overhead | ⭐⭐⭐ |
| 18.4 | Verified monomorphization — no boxing for generics | ⭐⭐⭐ |
| 18.5 | Array optimizations — small arrays on stack | ⭐⭐⭐ |
| 18.6 | Vtable elimination — direct dispatch for non-virtual | ⭐⭐⭐ |
| 18.7 | Return value optimization (RVO) — avoid copiis | ⭐⭐ |
| 18.8 | Devirtualization — speculative devirt for methods | ⭐⭐ |

### Async V3 — State Machine

| # | Task | Priority |
|---|------|----------|
| 9.1–5 | Replace thread pool with state machine V3 | ⭐⭐⭐ |
| 9.6–8 | Work-stealing scheduler | ⭐⭐⭐ |
| 9.9–11 | Non-blocking I/O (timers, signals, async read/write) | ⭐⭐ |

### Iterators Advanced

| # | Task | Priority |
|---|------|----------|
| 10.1–5 | Functional closuris (first-class fn pointers) | ⭐⭐⭐ |
| 10.6–9 | Lazy evaluation — `iter()` trait, lazy chains | ⭐⭐ |

### Alternative Backends

| Backend | Purpose |
|---------|---------|
| Cranelift | Faster compilation (debug mode), no LLVM dependency |
| WASM | Compile Kyle for browbe and WebAssembly targets |

---

## ✅ Completed — v0.6 Nuevo modelo ownership

| Fase | Description | Status |
|------|-------------|--------|
| 1 | Parser: `^` mutable, `&` borrow, move default | ✅ |
| 2 | Semantic: type inference for nuevo ownership | ✅ |
| 3 | HIR + MIR: lowering with move by defecto | ✅ |
| 4 | Borrow checker: use-after-move, one mut XOR many immut | ✅ |
| 5 | Benchmarks/packagis currentizados | ✅ |
| 6 | `^T` move antiguo eliminado | ✅ |

### Resumen de cambios v0.6

| Antis | Despues |
|-------|---------|
| `y = x` = borrow | `y = x` = **move** |
| `fn f(s: str)` = borrow | `fn f(s: str)` = **move** |
| `fn f(^s: str)` = move | Eliminado (move is default) |
| `fn f(&s: str)` = mutable | `fn f(s: &str)` = **borrow** |
| `x: &T = val` = mutable | `x: ^T = val` = **mutable** |
| `f(&x)` = mutable ref | `f(&x)` = **borrow**, `f(^&x)` = mutable borrow |

---

## 📊 Fasis de Madurez del Lenguaje (Benchmark gaps)

Featuris identificadas en benchmarks que Kyle necesita for be competitivo as language de bajo nivel:

| Feature | Importancia | Dependencia | ETA |
|---------|-------------|-------------|-----|
| **Native Arrays `[T, N]`** | ⭐⭐⭐⭐⭐ | Parbe + typeck + MIR + codegen | 🚀 **En progreso** |
| **Threads + Concurrency** | ⭐⭐⭐⭐⭐ | Runtime | 📅 Fase D |
| **Async/Await en Kyle** | ⭐⭐⭐⭐ | Runtime + compiler | 📅 Fase D |
| **HashMap completo** (String→any) | ⭐⭐⭐⭐ | Runtime | 📅 Fase C |
| **Networking TCP/UDP** | ⭐⭐⭐⭐ | Runtime + packagis | 📅 Fase 4-5 |
| **websocket/SSE** | ⭐⭐⭐ | Packagis | 📅 Fase 5 |
| **regex** | ⭐⭐⭐ | Package | 📅 Futuro |
| **Crypto** | ⭐⭐⭐ | Package | 📅 Futuro |
| **Compression** (gzip, brotli) | ⭐⭐⭐ | Package | 📅 Futuro |
| **PGO** (Profile Guided Optimization) | ⭐⭐⭐ | Toolchain | 📅 Futuro |
| **Cache Miss / IPC profiling** | ⭐⭐ | Toolchain | 📅 Futuro |
| **Arena/pool allocators** | ⭐⭐ | Runtime | 📅 Futuro |
| **Vectorization control** (LLVM) | ⭐⭐ | Compiler | 📅 Futuro |
| **Zero-cost Array/List/Dict syntax** | ⭐⭐⭐⭐⭐ | Parbe + typeck + MIR | 🚀 **En progreso** |

## Cambios de syntax (v0.6)

| Antis | Despues | Type |
|-------|---------|------|
| `[1, 2, 3]` | `[1, 2, 3]` | Array (before era list, ahora is array nativo) |
| `[1, 2, 3]` | `{1, 2, 3}` | List (cambia de `[]` a `{}`) |
| `list<T>` | `{T}` | List type (before `[T]`, ahora `{T}`) |
| — | `[T, N]` | Array type (nuevo, repite value) |
| `{"key": val}` | `{key: val}` | Dict literal (se can omitir quotes) |
| `fn f(^s: str)` = move | `y = x` = move by defecto | Ownership: move implicito |
| `y = x` = borrow | `fn f(s: &str)` = borrow | Ownership: `&` is borrow |
| `x: &T = v` = mutable | `x: ^T = v` = mutable | Ownership: `^` is mutable |
| `fn f(&s: str)` no existe | `fn f(s: ^&str)` = mut borrow | Ownership: `^&` compone |

### Nuevo modelo de ownership

| Syntax | Significado | Example |
|----------|-------------|---------|
| `x = v` | Variable inmutable (default) | `x = 42` |
| `x: ^T = v` | Variable mutable | `x: ^str = "hola"` |
| `y = x` | **MOVE** (no-Copy) | `t = s` → `s` invalido |
| `y = x.clone()` | **COPY** explicita | `t = s.clone()` → ambos vivos |
| `f(x)` | MOVE (owned) | `print(s)` → `s` se mueve |
| `f(&x)` | BORROW | `print(&s)` → `s` prestado |
| `f(^&x)` | MUT BORROW | `append(^&s)` → `s` prestado mutable |

Ver `docs/03-language-reference/ownership.md` for specification completa.

---

## 📈 Optimizacionis futuras (postergadas)

Improvements identificadas en benchmarks que cierran brecha with C/Rust:

| # | Improvement | Benchmarks | Impacto estimado | Esfuerzo |
|---|--------|-----------|-----------------|----------|
| 1 | **Register alloc for `^i32`/`^i64`** — LLVM mem2reg no promueve allocas simplis with multiplis BB. Solution: identificar `^i32`/`^i64` en codegen y emitir valueis LLVM directo without alloca | Fib (1.6× → 1.0×) | ⭐⭐⭐ | 1-2 days |
| 2 | **`list.reserve(n)` + batch push** — `reserve(n)` pre-asigna capacidad (✅ implemented). Batch push reduce N FFI calls a 1 | Primis (2.7× → 1.5×) | ⭐⭐ | 1 day |
| 3 | **Arrays `[T, N]` pass-by-reference** — Los arrays hoy are copy-by-value. `fn f(a: &[i64, N])` evitaria copiar 80KB en cada acceso. Necesita `&[T, N]` as type de parameter | Matmul (7.8× → 1.0×) | ⭐⭐⭐⭐⭐ | 3-4 days |
| 4 | **str_builder inline hints** — Marcar append with inlinehint for eliminar overhead de FFI call | Concat (1.1× → 0.5×) | ⭐ | 0.5 day |

### Notis de implementation

**#3 Arrays pass-by-reference** is improvement more importante (matmul is benchmark more representstivo de computo real). Hoy cada `a[i]` en `[i64, 10000]`:
1. (innecesario) Load del array completo (80KB)
2. GEP about resultado
3. Load del element

Con pass-by-reference:
1. GEP directo about pointer al array original
2. Load del element

Requiere nuevo `MirType::Slice` o modificar `MirInst::ArrayElemPtr` for aceptar referencias.

---

## 📋 Plan de completitud de types

Kyle must tener **todos typis importbefore as nativos** (no packages).
Solo HTTP/Postgres/SQLite are packages. El resto is infraestructura base.

### Fase 2: Migrar packagis a nativos

Cada type package → integration nativa requiere:
1. Type Kyle (`final class`) directamente en runtime
2. Builtins registrados en compiler (without `extern fn` manuales)
3. Sin `use X.Y` — disponiblis globalmente

| Package current | Type nativo | Runtime status |
|---------------|-------------|----------------|
| `use datetime` | `date_time` | ✅ `kyc_runtime/src/datetime.rs` |
| `use datetime.duration` | `duration` | ✅ en datetime.rs |
| `use date` | `Date` | ✅ `kyc_runtime/src/date.rs` |
| `use date.time` | `Time` | ✅ en date.rs |
| `use bytis.bytes` | `bytes` | ✅ `kyc_runtime/src/bytes.rs` |
| `use decimal` | `decimal` | ✅ `kyc_runtime/src/decimal.rs` |
| `use uuid` | `uuid` | ✅ `kyc_runtime/src/uuid.rs` |
| `use url` | `url` | ✅ `kyc_runtime/src/url.rs` |
| `use regex` | `regex` | ✅ `kyc_runtime/src/regex.rs` |
| `ky_getenv`/`ky_setenv` | `Env` | ✅ `kyc_runtime/src/string.rs` |

### Fase 3: Typis I/O nativos

| Type | Status | Necesita |
|------|--------|----------|
| `file` | ✅ | `final class file` with read/write/close/seek |
| `socket` | ❌ fd i32 | `final class socket` + listen/accept/connect |
| `path` | ❌ str | `final class path` + join/dirname/basename/exists |
| `json` | ❌ functions | `final class json` + parse/stringify methods |

### Fase 4: Collections faltantes

| Type | Status | Notis |
|------|--------|-------|
| `set<T>` | ✅ | Implementado |
| `Queue<T>` | ❌ | FIFO. Runtime simple (ring buffer) |
| `Stack<T>` | ❌ | LIFO. `{T}` with push/pop ya is stack |
| `slice` | ❌ | Vista de array existente `&[T]`. Necesario for pasar arrays without copiar |

### Fase 5: Concurrency nativa

| Type | Status | Notis |
|------|--------|-------|
| `channel<T>` | 🔶 | Runtime listo. Falta type Kyle generico |
| `mutex<T>` | ❌ | Para threads. Runtime Rust ya has |
| `AtomicI64` / `AtomicBool` | ❌ | Operacionis lock-free |
| `future<T>` | ❌ | Handle tipado for async |
| `iterator` | 🔶 | KlIter existe en runtime. Falta type Kyle |
| `select` | ❌ | Multiplexor de canalis |

### Fase 6: Smart pointers

| Type | Notis |
|------|-------|
| `box<T>` | Heap pointer simple. Ya existe `ky_alloc` for raw |
| `rc<T>` | Single-thread reference counting |
| `arc<T>` | Multi-thread atomic refcount |
| `weak<T>` | weak reference (evita ciclos rc/arc) |

---

## Status current del language

| Category | Completed | En progreso | Pending |
|-----------|:----------:|:-----------:|:---------:|
| Primitivos | 10/17 | 4 (u8-u64) | 3 (byte, void, never) |
| Compuestos | 9/15 | 1 (tuple) | 5 (set, Queue, Stack, Deque, LinkedList) |
| Ownership | 3/7 | 0 | 4 (box, rc, arc, weak) |
| Concurrency | 3/13 | 2 (channel, iterator) | 8 (future, select, mutex, RwLock, Atomic*2, Barrier, Condvar) |
| Especializados nativos | 0/15 | 10 (migrar de packages) | 5 (file, socket, path, json, big_int) |
| **Total** | **~25** | **~17** | **~25** |
