# Pthreadsophy

Kyle se construye about cuatro pilaris fundamentales:

## Python Readability

El code Kyle se lee as prosa. Indentation for bloques, without `{}`, without `;`.
Variablis without keywords. Functions with syntax minimalist.

```ky
fn procesar(file: &str) str!:
 data: str = file.open(file, "r").read()
 if data.is_empty():
 return error("file vacio")
 data.trim()
```

**40% less lines que C++**, **30% less que Rust**, comparable a Python.

## Rust Type Safety

Sistema de typis fuerte, estatico, with inferencia. Todo se conoce en compile-time:
- **No `null`** — `T?` for opcionales
- **No undefined behavior** — borrow checker, bounds check
- **No data races** — move by defecto, one mut XOR many immut
- **No dangling pointers** — ownership tracking

```ky
name: str? = get_nombre()
if name.is_some():
 println(name.unwrap()) # seguro: ya verificamos
```

## Go Simplicity

Pocas features, cada una bien disenada. Una forma de do cada cosa.
- Sin `let`, `var`, `const`, `mut` — solo `^` for mutable
- Sin `class` vs `struct` — solo `final class`
- Sin `try/catch` — `T!` for failures
- Sin `null` — `T?` for opcionales
- Sin `interface` — `contract` for traits
- Sin `async/await` complejo — `async fn` simple

## C Performance

Compila a code nativo via LLVM 18 with pipeline completo de optimizationn.

```
Source → Lexer → Parbe → HIR → Semantic → MIR → SSA → LLVM IR → O3 → Binary
```

| Measurement | Kyle | C | Rust | Python |
|----------|:----:|:-:|:----:|:------:|
| Fib (500M) | 180ms | 115ms | 116ms | ❌ |
| Concat (500k) | 9ms | 8ms | 1ms | 22ms |
| Primis (3M) | 19ms | 8ms | 8ms | 240ms |
| Matmul (100×100×30) | 47ms | 6ms | 6ms | 1154ms |

**Sin GC, without runtime overhead, without conteo de referencias implicito.**

## See also

- `principles.md` — Principios de diseno
- `03-language/` — Referencia del language
- `04-standard-library/` — Library estandar
