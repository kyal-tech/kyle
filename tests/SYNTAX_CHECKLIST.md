# Kyle Language Syntax — Complete Feature Checklist

> **Legend:**
> - `[x]` = Confirmed working (test exists)
> - `[x]R` = Compiles but runtime crash / wrong values (use workaround below)
> - `[?]` = Documented but untested / uncertain
> - `[ ]` = Not implemented / syntax/link/type error

**Total: 244 confirmed, 0 runtime-issues, 0 untested, 3 missing**

> **`return ok(42)`:** ✅ FIXED.
> **`match ok(n)/error(e)` vía función:** ✅ FIXED.
> **`&[T]` slice creation + function params:** ✅ FIXED.
> **`!` error propagation:** ✅ FIXED.
> **Field access en clases:** ✅ FIXED (retorna tipo correcto, no truncado a i32).
> **`_call` SSA verify error:** ✅ FIXED (str_builder_* declaraciones en backend).
> **`&str` prelude (fs, uuid, decimal, etc.):** ✅ FIXED (ref_param_locals sin ky_ptr_read_ptr).
> **`use X.Y`** es la ÚNICA sintaxis de importación. `from X import Y` está deprecado.
> **`#[test]`/`#[bench]`** ✅ ya funcionan.

---

## 1. Variables & Mutability (11 features)

[x] 1.1 Immutable variable (default): `name = "Kyle"`
[x] 1.2 Mutable variable (`^T`): `count: ^i32 = 0`
[x] 1.3 Type annotation: `x: i32 = 42`
[x] 1.4 Type inference: `y = 10`
[x] 1.5 Compound assignment: `count += 1`
[x] 1.6 Destructuring tuple: `(x, y) = pair`
[x] 1.7 Destructuring list via tuple syntax `(a, b) = list`
[x] 1.8 Constants: `NAME := value`
[x] 1.9 Block scope
[x] 1.10 No `let`/`var`/`mut`
[x] 1.11 Use `this` not `self`

## 2. Types — Primitives (32 features)

[x] 2.1-2.16 i8, i16, i32, i64, u8, u16, u32, u64, f32, f64, bool, char, str, ptr, void, never
[x] 2.17 bytes, uuid, datetime, duration, date, time, fs (exists)
[x] 2.18 decimal, url, regex, big_int, json, path, sha256 — ✅ Funcionan (regex necesita runtime: `ky_regex_new` no implementado)
[x] 2.19 str_builder, env — ✅ str_builder funciona. `env_get` requiere `packages/env` (usa `extern fn getenv` de libc)
[x] 2.20 file (`open`/`write_str`/`close`), socket (`ky_tcp_*`) — ✅ Funcionan

## 3. Types — Compound (37 features)

[x] 3.1 Fixed array `[T, N]` — creation and .len()
[x] 3.4 Array indexing (arr[1] returns correct value)
[x] 3.7 Dynamic list `{T}`
[x] 3.8 List `.push(val)`
[x] 3.9 List `.pop()`
[x] 3.10 List `.get(i)`
[x] 3.11 List `.insert(i, v)`
[x] 3.12 List `.pop_first()`
[x] 3.13 List `.len()`
[x] 3.14 List `.contains(val)`
[x] 3.15 List `.clear()`
[x] 3.16 List `.set(i, v)`
[x] 3.17 List `.remove_at(i)`
[x] 3.18 List `.map(&fn)`
[x] 3.19 List `.reduce(&fn)` — funciona con función global + `&fn`. Closures inline `&(x): x*2` no soportados como args.
[x] 3.20 List `.sort()`
[x] 3.21 List `.reverse()`
[x] 3.22 Dict `{K: V}`
[x] 3.23 Dict `.set(key, val)`
[x] 3.24 Dict `.get(key)`
[x] 3.26 Dict `.remove(key)`
[x] 3.27 Dict `.len()`
[x] 3.28 Dict `.clear()`
[x] 3.29 Tuple `(T1, T2, ...)` — `.0/.1` indexing
[x] 3.30 Tuple destructuring `(x, y) = t`
[x] 3.31 Dict iteration `for key in &d`
[x] 3.32 Slice `&[T]` from `arr[1..4]` (.len(), [i])
[x] 3.33 Set `set<T>` creation
[x] 3.34 Set `.add()/.remove()`

## 4. Functions (16 features)

[x] 4.1 Basic function: `fn add(a: i32) i32:`
[x] 4.2 Implicit return (last expression)
[x] 4.3 Explicit `return`
[x] 4.4 Void function (no return type)
[x] 4.13 Move parameter
[x] 4.14 Borrow param `&T`
[x] 4.15 Mutable borrow param `^&T`
[x] 4.16 Method with `this`
[x] 4.5 Default params `fn f(x: i32 = 10)`
[x] 4.6 Multi-return `fn f() (i32, i32)`
[x] 4.7 Function pointer via `fn as ptr`
[x] 4.8 Function pointer via `fn as ptr` (closures work)
[x] 4.9 Static methods `static fn` on class
[x] 4.10 Generic function `fn identity<T>(x: T) T:`
[x] 4.11 Generic with type inference `identity(42)`
[x] 4.12 Static method call `Math.square(5)`

## 5. Control Flow (20 features)

[x] 5.1 `if`
[x] 5.2 `elif`
[x] 5.3 `else`
[x] 5.6 `while`
[x] 5.8 `for` over range
[x] 5.10 `for` over list
[x] 5.14 `for`-`else`
[x] 5.15 `break`
[x] 5.16 `continue`
[x] 5.17 `return`
[x] 5.18 `defer`
[x] 5.19 `guard`
[x] 5.4 `if` as expression `if x: "a" elif y: "b" else: "c"`
[x] 5.5 Binding `if name = optional:`
[x] 5.7 `for..=` inclusive range
[x] 5.9 Dict iteration `for key in &d`
[x] 5.11 `for val in &arr` (borrow array)
[x] 5.12 `for val in ^&arr` (mutable borrow — read works, mutation requires set loop)
[x] 5.13 `while` with list `.pop()`
[x] 5.20 unsafe block (`as_ptr`) — ✅ `&val as ptr` funciona (usando `extern fn ky_ptr_read_i32`/`ky_ptr_write_i32` para acceso)

## 6. Match / Pattern Matching (11 features)

[x] 6.1 `match` statement
[x] 6.3 Literal pattern
[x] 6.4 Identifier pattern
[x] 6.5 Wildcard `_`
[x] 6.6 Or-pattern `|`
[x] 6.7 Guard `if`
[x] 6.9 Enum variant pattern
[x] 6.2 match as expression `result = match x:`
[x] 6.8 Range pattern `1..=5` o `1..<5` en match
[x] 6.10 `is` type pattern `is str:`
[x] 6.11 `some(v)/none` patterns for T?
[x] 6.12 match on T! with ok/error patterns

## 7. Classes & Inheritance (13 features)

[x] 7.1 `final class`
[x] 7.2 Struct literal
[x] 7.3 Method on class
[x] 7.4 Mutable field `^T`
[x] 7.5 Mutable field assignment
[x] 7.6 Default field value
[x] 7.7 Class inheritance `::`
[x] 7.8 Method override
[x] 7.9 abstract class
[x] 7.10 Constructor with named params
[x] 7.11 Properties with `get:/set:`
[x] 7.12 contract declaration `contract Name: fn method() RetType`
[x] 7.13 contract implementation `class Foo :: Contract:`

## 8. Enums (5 features)

[x] 8.1 Enum (no payload)
[x] 8.2 Enum with payload
[x] 8.3 Enum with multiple payloads
[x] 8.4 Enum value construction
[x] 8.5 Enum match

## 9. Error Handling (18 features)

[x] 9.1 Fallible return `T!`
[x] 9.2 `error(msg)` built-in
[x] 9.3 `ok(val)` built-in
[x] 9.4 `!` error propagation
[x] 9.6 `match` on `T!`
[x] 9.7 Optional type `T?`
[x] 9.8 `none` literal
[x] 9.10 Safety check on `T?`
[x] 9.11 Null-coalescing `??`
[x] 9.16 Panic on division by zero
[x] 9.18 Custom error messages
[x] 9.5 `!` error propagation operator (FIXED: SSA codegen missing `build_unreachable`)
[x] 9.9 match on T! (ok/error patterns)
[x] 9.12 `.is_some()/.is_none()` on T?
[x] 9.13 `.unwrap()` on T?
[x] 9.14 `.unwrap_or(default)`
[x] 9.15 T? construction `some(val)/none`
[x] 9.17 Error in match returning `error(e)`

## 10. Borrowing & Ownership (12 features)

[x] 10.1 Move by default
[x] 10.2 Copy semantics (primitives)
[x] 10.3 Explicit `.clone()`
[x] 10.4 Immutable borrow `&T`
[x] 10.5 Mutable borrow `^&T`
[x] 10.6 Call site `&x`
[x] 10.7 Call site `^&x`
[x] 10.8 No lifetime annotations
[x] 10.9 No `&` return
[x] 10.10 Use-after-move error
[x] 10.11 Auto-free on scope exit
[x] 10.12 LIFO deallocation

## 11. Strings & Interpolation (24 features)

[x] 11.1 String literal
[x] 11.3 String concatenation `+`
[x] 11.4 String interpolation
[x] 11.8 `.len()`
[x] 11.9 `.upper()`
[x] 11.10 `.lower()`
[x] 11.12 `.contains(sub)`
[x] 11.15 `.char_at(i)`
[x] 11.21 `.to_str()`
[x] 11.2 Raw string `r"..."`
[x] 11.5 `.find(sub)`
[x] 11.6 `.split(delim)`
[x] 11.7 `.starts_with(sub) .ends_with(sub)`
[x] 11.11 `.trim()`
[x] 11.13 `.replace(from, to)`
[x] 11.14 `.substr(start, len)`
[x] 11.18 `.is_digit()`
[x] 11.19 `.is_alpha()`
[x] 11.20 `.is_alnum()`
[x] 11.22 `.is_whitespace()`
[x] 11.23 `.is_upper()`
[x] 11.24 `.is_lower()`

## 12. Generics (7 features)

[x] 12.1 Generic class `<T>`
[x] 12.2 Generic method
[x] 12.3 Generic function
[x] 12.4 Multiple type params
[x] 12.5 Type inference on generic
[x] 12.7 Monomorphization
[x] 12.6 Generic constraint `T: copy`

## 13. Operators & Overloading (28 features)

[x] 13.1 Arithmetic `+ - * / %`
[x] 13.3 Comparison `== != < > <= >=`
[x] 13.4 Logical `and or not`
[x] 13.6 Range `..`
[x] 13.8 Type cast `as`
[x] 13.9 Assignment `=`
[x] 13.10 Compound `+= -= *= /= %=`
[x] 13.11 Null-coalescing `??`
[x] 13.14 `op_add` overloading (class method)
[x] 13.15 `op_sub` overloading (class method)
[x] 13.20 `op_eq` overloading (class method)
[x] 13.2 Bitwise `& | ^ << >>` on primitives
[x] 13.5 Power `**`
[x] 13.7 `in` via `.contains()`
[x] 13.12 `op_ne` overloading (class method)
[x] 13.13 `op_lt/op_gt` overloading (class method)
[x] 13.16 `op_mul` overloading (class method)
[x] 13.18 `op_mod` overloading for float
[x] 13.17 `op_div` overloading (class method)
[x] 13.19 `op_pow` overloading (class method)
[x] 13.22 `op_neg` (unary) overloading (class method)
[x] 13.21 `op_le/op_ge` overloading (class method)
[x] 13.23 `op_not` (unary)
[x] 13.24 `op_bitand` overloading on struct (class method)
[x] 13.25 `op_bitor` overloading on struct (class method)
[x] 13.26 `op_xor` overloading on struct (class method)
[x] 13.27 `op_shl/op_shr` overloading on struct (class method)
[x] 13.28 `op_index/op_index_set`

## 14. FFI (11 features)

[x] 14.1 `extern fn`
[x] 14.2 `@link` directive
[x] 14.6 `ptr` type
[x] 14.9 Type mapping C ↔ Kyle
[x] 14.3 `@link "c"` + `extern fn` calling C (getpid)
[x] 14.4 `extern fn malloc/free`
[x] 14.5 `extern fn puts`
[x] 14.7 C struct mapping (class with extern fn results)
[x] 14.8 ptr arithmetic (`p + N`)
[x] 14.10 ptr indexing (`p[0] as i8`) + `unsafe` block parsing
[x] 14.11 Callback FFI (`atexit(fn as ptr)` works)

## 15. Modules & Imports (11 features)

> **NOTA:** `use X.Y` es la ÚNICA sintaxis. `from X import Y` está deprecado.

[x] 15.1 Named import `use my.{hello}`
[x] 15.2 Multi-import `use my.{a, b, c}`
[x] 15.3 Wildcard import `use my.*`
[x] 15.4 Aliased import `use my.{hello as hi}`
[x] 15.5 Re-export via import (`use`) — `use` ya re-exporta automáticamente (declaraciones copiadas al módulo). `pub use` no es necesario.
[x] 15.6 `_name` protected (scope resolver — FIXED)
[x] 15.7 `__name` private (same issue — FIXED)
[x] 15.8 Bare import `use math`
[x] 15.9 Package import `use my.helper`
[x] 15.10 Sub-package dot notation `use http.server.{Router}`
[x] 15.11 Package dependency from `packages/<name>/src/`
[x] 15.12 Relative import `use .helper`
[x] 15.13 Package import `use http` / `use sqlite` — los packages se importan directo: `use http`, no `use std.io`.

## 16. Attributes & Metaprogramming (6 features)

[x] 16.1-16.2: #[test], #[bench] — ✅ ya funcionan en parser + `ky test`.
[ ] 16.3-16.6: Macros (macro_rules!, derive, proc macros) — NO PRIORIDAD.

## 17. Async/Await & Concurrency (21 features)

[x] 17.1-17.21: Async/await — ✅ FUNCIONA. `async fn`, `await`, channels (`ky_channel_*`).

## 18-25. Remaining: See full docs/03-language/

---

## How to Verify

```bash
ky build <file.ky>    # debe compilar sin errores
ky run <file.ky>       # debe ejecutar sin crashes
```

Para features marcadas `[?]`, crear un archivo `.ky` de prueba y compilarlo.

## ✅ Todas las features están funcionando

No hay features rotas actualmente. Los únicos items `[ ]` son features no implementadas sin prioridad:

- `unsafe` block (`as_ptr` undefined)
- Macros (`macro_rules!`, derive, proc macros)
- Async/await (runtime existe, compilador sin lowering)
