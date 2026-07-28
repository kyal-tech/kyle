# FAQ

> Questions frecuentis about Kyle.

## Generales

### Is Kyle a high or low level language?

**Bajo nivel.** Compila a nativo via LLVM, has control de memory manual
(ownership, borrow checker), pointers raw, FFI directo with C. Pero with syntax
simple y legible as Python.

### �Does Kyle compete with Rust?

Si, en sentido de be un language de sistemas seguro y rapido. Pero Kyle
prioriza simplicidad sintactica about exhaustividad del type system.
Menos features, less complejidad.

### �Does Kyle have a garbage collector?

**No.** La memory se gestiona using ownership (move by defecto) y 
borrow checker inserta `ky_free` automaticamente.

## Syntax

### �Why snake_case and not camelCase?

snake_case is more legible for code with nombris largos y is consistente
with philosophy de "without ruido sintactico".

### �Why is there no `let`/`var`/`mut`?

Para reducir ruido. La declaration is `name = value`. La mutabilidad se
marca with `^`: `x: ^i32 = 0`.

### �Why `^` for mutable and `&` for borrow?

`^` is un sigilo minimalist que no compite with operadoris existentes.
`&` for borrow is familiar for programadoris Rust.

## Rendimiento

| Benchmark | Kyle vs C | Gap | Causa principal |
|-----------|:---------:|:---:|-----------------|
| Fib | 1.6× | Register alloc | Optimizar `^i32` en codegen |
| Concat | 1.1× | FFI call overhead | strBuilder inline hints |
| Primis | 2.7× | List push overhead | `reserve()` + batch push |
| Matmul | 7.8× | List get/set calls | Arrays nativos pass-by-ref |

## Paquetes

### �Why are DateTime, Regex, UUID native and not packages?

Porque are typis base que cualquier application necesita. Solo HTTP, SQLite y
PostgreSQL are packagis porque are protocolos/basis de data especificos.

### �Where are the runtime files?

En `crates/kyc_runtime/src/`. 3350 lines de Rust, 88 functions `extern "C"`.

## See also

- `pthreadsophy.md` — Philosophy del language
- `architecture.md` — Arquitectura del ecosistema
