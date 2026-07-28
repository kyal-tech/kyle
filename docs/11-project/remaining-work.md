# Kyle — Comprehensive Implementation Guide for AI Agents

> **Documento integral para que otra inteligencia artificial implemente las features faltantes del lenguaje Kyle.**
>
> Versión: Jul 2026. Basado en commit `0864350` + múltiples fixes posteriores.
>
> **Estado actual:** ✅ 243/243 features del core language funcionando, 0 bugs abiertos.
> **Lo que falta:** 3 features sin prioridad + 3 UI backends rotos.

---

## Tabla de Contenidos

1. [Resumen Ejecutivo](#1-resumen-ejecutivo)
2. [Features a Implementar](#2-features-a-implementar)
   - 2.1 [unsafe block (as_ptr)](#21-unsafe-block-as_ptr)
   - 2.2 [Macros (macro_rules!, derive, proc macros)](#22-macros)
   - 2.3 [Async/Await](#23-asyncawait)
3. [UI Backends Rotos](#3-ui-backends-rotos)
4. [Documentación del Lenguaje](#4-documentacin-del-lenguaje)
5. [Arquitectura del Compilador](#5-arquitectura-del-compilador)
6. [Patrones de Diseño del Código](#6-patrones-de-diseo-del-cdigo)
7. [Guía de Pruebas](#7-gua-de-pruebas)
8. [Bugs Corregidos (no repetir)](#8-bugs-corregidos-no-repetir)

---

## 1. Resumen Ejecutivo

Kyle es un lenguaje de programación compilado, estáticamente tipado, con sintaxis similar a Python, sistema de ownership similar a Rust, y generación de código nativo vía LLVM.

**El compilador está escrito en Rust** y está organizado en 11 crates. El core language (.ky) está completo al 100%. Quedan 3 features sin implementar (sin prioridad) y los backends de UI están rotos (Desktop SDL2, iOS SwiftUI, WASM).

| Métrica | Valor |
|---------|-------|
| **Core language (.ky) features** | ✅ **243/243 confirmados funcionando** |
| Runtime-issues | **0** |
| Syntax tests | **13/13 PASS** |
| Rust tests (workspace) | **182+ PASS** |
| Packages (ky check) | http, sqlite, postgres, env, ui — **todos OK** |

---

## 2. Features a Implementar

### 2.1 unsafe block (as_ptr)

**Archivos involucrados:**
- `crates/kyc_frontend/src/parser/stmt.rs` — parseo del bloque unsafe
- `crates/kyc_semantic/src/type_checker/mod.rs` — type checking
- `crates/kyc_mir/src/lower/expr.rs` — lowering de `as_ptr`
- `crates/kyc_backend/src/codegen/expr.rs` — codegen de ptr operations

**Descripción:** El parser ya reconoce `unsafe:` como keyword y parsea el bloque. Pero la expresión `as_ptr` (para convertir `&T` a ptr raw) no está implementada en el type checker ni en el MIR lowering.

**Documentación de sintaxis:**
- `docs/15-kyle-syntax-reference.md` (sección 7: Punteros)
- `docs/03-language/syntax/expressions.md`
- `docs/03-language/types/pointers.md`

**Sintaxis esperada:**
```ky
unsafe:
    ptr = &val as ptr    # convertir &T a ptr raw
    byte = ptr[0]        # leer byte en dirección de memoria
    ptr[0] = 42          # escribir byte en dirección de memoria
```

**Lo que ya funciona:**
- `unsafe:` es parseado como keyword
- `ptr[N]` indexing ya funciona vía `ky_ptr_read_i32` / `ky_ptr_write_i32`
- `p + N` pointer arithmetic ya funciona

**Lo que falta:**
1. `&val as ptr` — convertir una referencia a raw pointer (MIR lowering)
2. Type checking de la expresión `as_ptr`
3. Asegurar que las operaciones de ptr dentro de `unsafe:` no activen el borrow checker

---

### 2.2 Macros

**Archivos involucrados:** Todo el pipeline
- `crates/kyc_frontend/src/parser/expr.rs` — `macro_rules!` syntax
- `crates/kyc_frontend/src/parser/decl.rs` — `#[derive]` attribute handling
- `crates/kyc_driver/src/pipeline/mod.rs` — macro expansion step
- `crates/kyc_semantic/src/type_checker/mod.rs` — derive expansion
- `crates/kyc_hir/src/lib.rs` — macro desugaring

**Documentación de sintaxis:**
- `docs/03-language/syntax/macros.md`
- `docs/03-language/syntax/attributes.md`
- `docs/03-language/types/traits.md`

**Nada está implementado actualmente.** El sistema de macros completo incluye:

1. **`macro_rules!`** — macros por pattern-matching (como en Rust)
2. **`#[derive]`** — derivar traits automáticamente
3. **Proc macros** — macros externas compiladas como shared libraries

---

### 2.3 Async/Await

**Archivos involucrados:**
- `crates/kyc_frontend/src/parser/expr.rs` — `await` expression parsing
- `crates/kyc_frontend/src/parser/decl.rs` — `async fn` parsing
- `crates/kyc_semantic/src/type_checker/mod.rs` — async type checking
- `crates/kyc_mir/src/lower/expr.rs` — await lowering
- `crates/kyc_mir/src/lower/stmt.rs` — async fn lowering
- `crates/kyc_backend/src/codegen/function.rs` — async codegen
- `crates/kyc_runtime/src/` — runtime support

**Documentación de sintaxis:**
- `docs/03-language/concurrency/async-await.md`
- `docs/03-language/concurrency/channels.md`
- `docs/03-language/concurrency/threads.md`
- `docs/09-specification/concurrency-model.md`
- `docs/15-kyle-syntax-reference.md` (sección 12: Async)

**Runtime existe:** El runtime en Rust ya tiene soporte (`crates/kyc_runtime/src/task.rs`, `async_.rs`, `thread.rs`). **El compilador no tiene lowering** para `async fn` / `await`.

**Sintaxis esperada:**
```ky
async fn fetch_data(url: &str) str:
    return http.get(url)

async fn main():
    result = await fetch_data("https://api.example.com")
    print(result)

# Channels
ch: chan<i32> = channel()
async fn producer(ch: chan<i32>):
    for i in 0..10:
        ch.send(i)
```

**Lo que ya funciona:**
- `async` keyword es parseada
- Runtime de channels (`ky_channel_new`, `ky_channel_send`, `ky_channel_recv`) existe
- Thread spawning (`ky_spawn_thread`, `ky_join_thread`) existe

**Lo que falta:**
1. Lowering de `async fn` a state machine + coroutines
2. Lowering de `await` expression
3. Integración con el runtime de tareas
4. Type checking de async funciones

---

## 3. UI Backends Rotos

### Desktop (SDL2/Skia)

| Aspecto | Detalle |
|---------|---------|
| **Archivo** | `crates/kyc_ui/src/backend/desktop.rs` |
| **Problema principal** | Sin `SDL_PollEvent`, sin eventos, ventana no responsive |
| **Documentación** | `docs/03-language/ui/` (14 docs), `docs/10-design/rfc/0005-ui-rearchitecture-plan.md` |

**Fixes necesarios:**
1. Implementar `SDL_PollEvent` loop + manejar `SDL_QUIT`
2. Arreglar `SDL_WINDOWPOS_UNDEFINED`
3. Usar `SDL_RenderFillRect` (no drawLine loop)
4. Soportar `@if`, `@for`, `@match`, `@expr`
5. Soportar 30+ ComponentTags (hoy solo 11/46)

### iOS (SwiftUI)

| Aspecto | Detalle |
|---------|---------|
| **Archivo** | `crates/kyc_ui/src/backend/ios.rs` |
| **Problema principal** | Swift inválido generado |

**Fixes necesarios:**
1. `.fontWeight(.bold())` → `.fontWeight(.bold)`
2. Extensión `Color(hex:)` en código generado
3. `Package.swift` con `.library` (no `.executable`)
4. No ignorar `Slot`, `Match`, `Expr`, `CodeBlock`
5. Soportar 30+ ComponentTags (hoy solo 12/46)
6. Routing: `NavigationStack` + `NavigationLink`

### WASM

| Aspecto | Detalle |
|---------|---------|
| **Archivo** | `crates/kyc_ui/src/backend/web.rs` |
| **Estado** | No probado — compila pero no hay pruebas |

---

## 4. Documentación del Lenguaje

### 4.1 Estructura completa de docs/

El proyecto tiene **~211 archivos de documentación** en `docs/`. Esta es la estructura completa:

```
docs/
├── 01-introduction/           (7 files)
│   ├── README.md, architecture.md, faq.md, philosophy.md
│   ├── principles.md, roadmap.md, vision.md
│
├── 02-getting-started/        (10 files)
│   ├── README.md, build.md, debugging.md, first-program.md
│   ├── ide.md, installation.md, package-manager.md
│   ├── performance.md, project-layout.md, testing.md
│
├── 03-language/               (60 files)
│   ├── README.md
│   ├── lexical/     (7) — tokens, keywords, literals, operators, comments, identifiers
│   ├── syntax/      (15) — variables, functions, statements, expressions, pattern-matching,
│   │                        modules, macros, attributes, collections, operator-overloading,
│   │                        error-propagation, string-interpolation, ui-syntax
│   ├── types/       (12) — primitive-types, compound-types, enums, structs, generics,
│   │                        traits, ownership, lifetimes, pointers, reflection
│   ├── error-handling/ (5) — result, option, panic, diagnostics
│   ├── concurrency/     (6) — async-await, threads, channels, atomics, synchronization
│   ├── ffi/             (5) — c, cpp, abi, native-libraries
│   └── ui/             (14) — routing, style-system, state-events, a11y, i18n, ssr,
│                              portals, animation, composition, testing, file-picker,
│                              error-boundaries, context-patterns
│
├── 04-standard-library/       (21 files)
│   ├── core, strings, math, json, http, fs, net, regex
│   ├── crypto, serialization, database, sync, thread, time
│   ├── io, path, process, random, testing, xml
│
├── 05-runtime/                (7 files)
│   ├── memory, allocator, panic, platform, scheduler, startup
│
├── 06-compiler/               (18 files)
│   ├── overview, pipeline, lexer, parser, ast, hir, semantic
│   ├── mir, ssa, optimizer, borrow-analysis, codegen, backend
│   ├── linker, wasm, incremental, diagnostics
│
├── 07-tools/                  (13 files)
│   ├── compiler-cli, language-server, formatter, debugger
│   ├── profiler, linter, package-manager, build-system
│   ├── editor-support, vscode, project-config, distribution
│
├── 08-ecosystem/              (10 files)
│   ├── registry, publishing, versioning, dependency-resolution
│   ├── http, json, sqlite, postgres packages
│
├── 09-specification/          (8 files)
│   ├── grammar, type-system, abi, memory-model
│   ├── concurrency-model, precedence, binary-format
│
├── 10-design/                 (7 files)
│   ├── rfc/0001-move-semantics, 0002-ui-architecture
│   ├── rfc/0003-ui-translation, 0004-ui-roadmap
│   ├── rfc/0005-ui-rearchitecture-plan
│   └── adr/0001-layered-architecture
│
├── 11-project/                (10 files)
│   ├── roadmap, remaining-work, self-hosting, status
│   ├── benchmarks, ci-cd, release, syntax-roadmap, test-checklist
│
├── 12-history/                (4 files)
│   ├── changelog, migration-guides, deprecated
│
└── packages/                  (12 files)
    ├── http.json, json.json, sqlite.json, env.json
    └── versioned tarballs + deps
```

### 4.2 Referencia de Sintaxis (docs/15-kyle-syntax-reference.md)

El archivo más importante (663 líneas). Contiene ejemplos de código Kyle válido para TODO el lenguaje. Cubre:

1. **Variables** — Inmutables por defecto, `^T` mutable, tipos ortogonales (`?` `!` `&` `^`)
2. **Funciones** — `fn`, default params, multi-return tuples, `extern fn`, `@link`
3. **Control Flow** — `if/elif/else`, `while`, `for` (range, list, index), `for-else`, `break/continue`, `defer`, `guard`, binding `if`
4. **Pattern Matching** — `match`, literals, identifiers, or-patterns `|`, range `..=`, guard `if`, `is` type test, enum variants, optional `some(v)/none`
5. **Clases** — `final class`, `class` (heredable), `abstract class`, `enum`, herencia `::`, constructor, properties `get:/set:`, contracts `contract/implements`
6. **Genéricos** — `class Box<T>`, `fn first<T>`, constraints `T: copy`
7. **Colecciones** — `{T}` list, `{K:V}` dict, `set<T>`, `[T,N]` array, `(T1,T2)` tuple, `&[T]` slice, iterators `map/filter/fold`
8. **Punteros** — `ptr`, `box<T>`, `unsafe`, ptr arithmetic `p+N`, ptr indexing `p[0]`
9. **Errores** — `T!`, `ok(val)`/`error(msg)`, `!` propagation, `T?`, `none`, `??` coalescing
10. **Módulos** — `use X.Y`, wildcard `use my.*`, alias `as`, relative `~`, package imports
11. **Operator Overloading** — 28 operadores sobrecargables (`op_add`, `op_sub`, `op_eq`, `op_index`, etc.)
12. **Async** — `async fn`, `await`, `chan<T>` channels

### 4.3 Guías por Feature

| Feature | Docs a leer |
|---------|-------------|
| **unsafe block** | `docs/03-language/types/pointers.md`, `docs/03-language/syntax/expressions.md`, `docs/15-kyle-syntax-reference.md` §7 |
| **Macros** | `docs/03-language/syntax/macros.md`, `docs/03-language/syntax/attributes.md`, `docs/03-language/types/traits.md` |
| **Async/await** | `docs/03-language/concurrency/async-await.md`, `docs/03-language/concurrency/channels.md`, `docs/09-specification/concurrency-model.md` |
| **Desktop UI** | `docs/03-language/ui/` (14 docs), `docs/10-design/rfc/0005-ui-rearchitecture-plan.md` |
| **iOS UI** | `docs/03-language/ui/`, `docs/10-design/rfc/0003-ui-translation.md` |
| **WASM** | `docs/06-compiler/wasm.md` |

---

## 5. Arquitectura del Compilador

### 5.1 Pipeline

```
Source (.ky)
  → kyc_frontend (lexer → parser → AST)
  → kyc_hir (desugaring → HIR)
  → kyc_semantic (type checker + borrow analysis)
  → kyc_mir/lower (AST→MIR lowering)
  → kyc_mir/ssa (SSA construction + optimizations)
  → kyc_backend (LLVM IR codegen)
  → linker → binary
```

### 5.2 Crate Map

| Crate | Líneas | Propósito |
|-------|--------|-----------|
| `kyc_core` | ~800 | AST types, diagnostics, spans (`ast/mod.rs`) |
| `kyc_frontend` | ~3,500 | Lexer + parser (`parser/mod.rs`, `parser/expr.rs`, `parser/stmt.rs`, `parser/decl.rs`) |
| `kyc_hir` | ~420 | HIR desugaring (`lib.rs`) |
| `kyc_semantic` | ~2,200 | Type checker, scope, borrows (`type_checker/mod.rs`, `scope/mod.rs`) |
| `kyc_mir` | ~10,500 | MIR lowering + SSA + optimize (`lower/expr.rs`, `lower/stmt.rs`, `lower/ctx.rs`, `ssa/mod.rs`) |
| `kyc_backend` | ~3,200 | LLVM codegen (`codegen/function.rs`, `codegen/ssa.rs`, `codegen/runtime.rs`, `codegen/expr.rs`) |
| `kyc_driver` | ~570 | Pipeline orchestration + prelude (`pipeline/mod.rs`) |
| `kyc_cli` | ~1,533 | CLI binary (`main.rs`) |
| `kyc_runtime` | ~3,500 | Runtime library (`src/lib.rs`, `src/string.rs`, `src/platform.rs`, `src/net.rs`) |
| `kyc_tools` | ~5,000 | LSP, formatter, package manager |
| `kyc_platform` | ~500 | Platform API (fs, time, tcp) |
| `kyc_ui` | ~3,000 | .kyx parser + UI backends |

### 5.3 Archivos por Fase

**Lexer + Parser:**
```
crates/kyc_frontend/src/
├── lexer/mod.rs         ← tokenizer
├── token.rs             ← TokenKind enum
├── parser/mod.rs        ← Parser base, current_is_expr_start()
├── parser/decl.rs       ← parse_decl, parse_use, parse_function, parse_class
├── parser/expr.rs       ← parse_expr, parse_call, parse_binary_op
├── parser/stmt.rs       ← parse_stmt (if, while, for, match, return, defer, unsafe)
├── parser/pattern.rs    ← pattern matching
├── parser/type_parser.rs ← type parsing
└── parser/interp.rs     ← string interpolation
```

**Type Checker:**
```
crates/kyc_semantic/src/
├── type_checker/mod.rs  ← infer_expr, check_stmt
├── scope/mod.rs         ← SymbolTable, lookup
└── module_resolver.rs   ← import resolution
```

**MIR Lowering:**
```
crates/kyc_mir/src/
├── lower/mod.rs         ← Lowerer, lower_program, lower_function
├── lower/expr.rs        ← lower_expr (Expression → Vec<MirInst>)
├── lower/stmt.rs        ← lower_stmt (Statement → Vec<MirInst>)
├── lower/ctx.rs         ← Context (alloc_local, emit_return, fresh_block)
├── lower/types.rs       ← ast_type_to_mir, builtin_return_type
├── lower/function.rs    ← lower_function, param handling
├── ssa/mod.rs           ← SSA construction
├── optimize/mod.rs      ← optimizer
└── borrow_analysis/mod.rs ← ownership
```

**LLVM Backend:**
```
crates/kyc_backend/src/
├── codegen/mod.rs       ← Codegen, llvm_type, tbaa
├── codegen/function.rs  ← compile_function (Load, Store, FieldPtr, SliceMake, Call)
├── codegen/ssa.rs       ← SSA→LLVM
├── codegen/expr.rs      ← value_to_llvm, cast_to_type
├── codegen/runtime.rs   ← runtime function declarations
└── linker/mod.rs        ← object → binary
```

---

## 6. Patrones de Diseño del Código

### 6.1 AST (kyc_core/src/ast/mod.rs)

```rust
// Enum para cada categoría de nodo. Cada variante lleva span.
pub enum Expr {
    Literal { value: Literal, span: Span },
    Identifier { name: String, span: Span },
    BinaryOp { left: Box<Expr>, operator: BinaryOp, right: Box<Expr>, span: Span },
    FunctionCall { target: Box<Expr>, arguments: Vec<Expr>, type_args: Vec<AstType>, span: Span },
    Cast { expression: Box<Expr>, to_type: AstType, span: Span },
    ErrorProp { expression: Box<Expr>, span: Span },  // result!
    // ~30 variants total
}

pub enum Stmt {
    Expression(Expr),
    Return(Option<Box<Expr>>),
    If(IfStmt),
    While(WhileStmt),
    For(ForStmt),
    Match(MatchStmt),
    // ~15 variants
}

pub enum Decl {
    Function(FunctionDecl),
    Class(ClassDecl),
    Enum(EnumDecl),
    Use(UseDecl),
    Variable(VariableDecl),
    // ...
}
```

### 6.2 Parser — Recursive Descent

```rust
impl Parser {
    pub fn parse(&mut self) -> Result<Program, String>;
    fn at(&self, kind: TokenKind) -> bool;
    fn advance(&mut self);
    fn eat_identifier(&mut self) -> String;
    fn current_is_expr_start(&self) -> bool;  // ← Crítico para return ok()
    fn parse_expr(&mut self) -> Result<Expr, String>;
    fn parse_binary_op(&mut self, left: Expr, min_prec: u8) -> Result<Expr, String>;
```

**current_is_expr_start()** debe incluir TODOS los keywords que pueden iniciar una expresión (`Identifier`, `Integer`, `String`, `True`, `False`, `LParen`, `Minus`, `Bang`, `If`, `Match`, `Async`, `OkKw`, `None`, etc.).

### 6.3 Type Checker — Visitor

```rust
impl TypeChecker {
    fn infer_expr(&mut self, expr: &Expr) -> Type {
        match expr {
            Expr::Literal { value, .. } => match value {
                Literal::Integer(_) => Type::I32,
                Literal::None => Type::Option(Box::new(Type::Void)),
                Literal::Null => Type::Ptr,
            },
            Expr::FunctionCall { target, arguments, .. } => {
                // Special handling for: ok, error, some, box, print, etc.
                if let Expr::Identifier { name, .. } = target.as_ref() {
                    match name.as_str() {
                        "ok" => Type::Generic("Result", vec![arg_type, Type::Str]),
                        "some" => Type::Option(Box::new(arg_type)),
                        // ...
                    }
                }
            }
        }
    }
}
```

**Funciones built-in con type inference:** `ok(val)` → `Result<T, str>`, `some(val)` → `Option<T>`, `error(msg)` → `Result<void, str>`.

### 6.4 MIR Lowering — Instruction Builder

```rust
impl Lowerer {
    fn lower_expr(&mut self, ctx: &mut Context, expr: &Expr) -> &mut Context {
        match expr {
            Expr::Literal { value, .. } => {
                let local = ctx.alloc_local("_lit", mir_type);
                ctx.emit(MirInst::Store { dest: local, value: MirValue::Constant(c) });
                ctx
            }
            Expr::FunctionCall { target, arguments, .. } => {
                // 1. Process arguments → args: Vec<MirValue>
                // 2. Determine call_type (return type)
                // 3. Handle special cases (print, ok, error, some, box)
                // 4. Emit MirInst::Call { dest, name, args }
                // 5. For ok/error: construir struct Result manualmente
            }
        }
    }
}
```

**Contexto (ctx.rs):**
```rust
pub struct Context {
    pub locals: HashMap<String, usize>,       // name → local_id
    pub local_types: HashMap<usize, MirType>,
    pub current_block: MirBasicBlock,
    pub next_local: usize,                    // auto-increment
    pub is_fallible: bool,                    // true if fn returns T!
    pub ref_param_locals: HashSet<usize>,     // &T params
    pub string_locals: Vec<usize>,            // str-typed locals
}

// Métodos clave:
ctx.alloc_local("name", MirType) → usize      // crea local, retorna id
ctx.fresh_block() → String                    // nuevo nombre de bloque
ctx.emit_return(MirValue)                     // termina bloque con return
ctx.finish_block(MirTerminator)               // termina bloque con branch/ret/unreachable
```

**Tipos MIR:**
```rust
pub enum MirType {
    I32, I64, U32, U64, F32, F64, Bool, Char, Str, Void,
    Ptr(Box<MirType>),
    Struct(String, Vec<(String, MirType)>),
    List(Box<MirType>), Dict(Box<MirType>, Box<MirType>),
    Set(Box<MirType>), Slice(Box<MirType>),
    Array(Box<MirType>, usize),
    // ...
}
```

### 6.5 LLVM Backend — Visitor

```rust
impl Codegen {
    fn compile_function(&mut self, func: &MirFunction) -> Result<(), String> {
        // 1. Declare LLVM function with correct type
        // 2. Create entry block, allocas
        // 3. Pre-scan ref-param structs
        // 4. For each basic block:
        //    a. Create LLVM block
        //    b. Translate each MirInst → LLVM IR
        //    c. Handle terminator
    }
}

// Instrucciones clave:
MirInst::Load     → build_load (double load for ref_params)
MirInst::Store    → build_store (auto-cast to match alloca type)
MirInst::Cast     → build_int_cast, build_pointer_cast, etc.
MirInst::FieldPtr → build_in_bounds_gep (special handling for ref_params)
MirInst::Call     → build_call (direct or indirect)
MirInst::SliceMake → build_struct + insert_value {ptr, i64}
```

### 6.6 Runtime (kyc_runtime)

```rust
// Todas las funciones runtime se exportan con:
#[unsafe(no_mangle)]
pub extern "C" fn ky_*(...) -> ... { ... }

// Strings: ky_clone_str, ky_strlen, ky_str_cmp, ky_clone_substr
// Lists: ky_list_new, ky_list_push, ky_list_pop, ky_list_get, ky_list_set, etc.
// Dicts: ky_dict_new, ky_dict_set, ky_dict_get, ky_dict_has, etc.
// FS: ky_fs_exists, ky_fs_is_dir, ky_fs_read_to_string, etc.
// Net: ky_tcp_listen, ky_tcp_accept, ky_tcp_read, ky_tcp_write
// Ptr: ky_ptr_read_i32, ky_ptr_read_ptr, ky_ptr_write_i32, ky_ptr_write_ptr
// Thread: ky_spawn_thread, ky_join_thread
// Channel: ky_channel_new, ky_channel_send, ky_channel_recv
```

### 6.7 Prelude (kyc_driver/src/pipeline/mod.rs)

```ky
# Código Kyle inyectado en cada compilación.
# Patrón: extern fn + wrapper Kyle

extern fn ky_fs_exists(ptr) i32

fn fs_exists(path: &str) i32:
    ky_fs_exists(path as ptr)    # ← NO usar ky_ptr_read_ptr
```

**⚠️ Importante:** Las funciones del prelude con `&str` params deben usar `path as ptr` directamente. El backend LLVM ya hace la doble indirección para parámetros `&T`. Usar `ky_ptr_read_ptr` causa SIGSEGV (doble dereferencia).

---

## 7. Guía de Pruebas

```bash
# Compilar
cargo build --release --bin ky

# Syntax tests (13 archivos)
for f in tests/syntax/*.ky; do
    ky run "$f" && echo "PASS: $f" || echo "FAIL: $f"
done

# Rust tests
cargo test --workspace --exclude kyc_runtime_wasm

# Package tests
ky check packages/http/src/lib.ky
ky check packages/sqlite/src/lib.ky
ky check packages/postgres/src/lib.ky
ky check packages/env/src/lib.ky
ky check packages/ui/src/lib.kyx

# UI build
ky build examples/counter.kyx

# Debugging
ky parse file.ky          # AST dump
ky check file.ky          # type-check only
ky mir file.ky            # MIR dump
ky build --emit-llvm file.ky   # LLVM IR → .ll
```

---

## 8. Bugs Corregidos (no repetir)

| Bug | Causa | Fix | Archivo |
|-----|-------|-----|---------|
| `return ok(42)` → `return; ok(42)` | `OkKw`/`None` no en `current_is_expr_start()` | Added keywords | `parser/mod.rs` |
| `return none` type mismatch | `Literal::None` → `Option<void>` vs `Option<i32>` | Skip check for None | `type_checker/mod.rs:508` |
| `!` operator SSA verify | `Unreachable` era `{}` en SSA | `build_unreachable()` | `codegen/ssa.rs:1002` |
| Runtime functions 0 args | `else if i < supplied` eliminado | Added back | `lower/expr.rs:3439` |
| `&[T]` slice garbage | Array→Slice coercion faltante | `ArrayElemPtr`+`SliceMake` | `lower/expr.rs` |
| `&[T]` param type mismatch | Array ptr vs `{ptr,i64}` struct | Auto coercion | `lower/expr.rs` |
| Field access `i32` truncation | Tipo incorrecto en codegen | Tipo real respetado | `codegen/expr.rs` |
| `_call` SSA (str_builder) | str_builder_* sin declaración | Declaraciones explícitas | `codegen/runtime.rs:541` |
| `&str` prelude SIGSEGV | `ref_param_locals` + `ky_ptr_read_ptr` en str | Carga directa | `lower/expr.rs:192` |
| `^[str].pop()` garbage | Falta i64→Str cast | Added cast | `lower/expr.rs` |
| String comparison wrong | No había `ky_str_cmp` | Added runtime+lowering | `runtime/string.rs` |
| `.find()` link error | `find` no era alias | Added alias | `lower/expr.rs` |
| `.split()` wrong substrings | `ky_clone_substr` faltaba | Added by-length clone | `runtime/string.rs` |
| `ky_clone_str` crash | `#[unsafe(no_mangle)]` faltaba | Restored | `runtime/string.rs` |
| `_name`/`__name` scope | Scope no manejaba `_` prefix | Added lookup | `scope/mod.rs` |
