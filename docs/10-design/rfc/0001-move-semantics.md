# RFC-0001: Move Semantics

**Status:** Implemented (v0.5 → v0.6 currentizado) 
**Date:** 2026-06-01 
**Última currentizacion:** v0.6 — move by defecto, `^` = mutable, `&` = borrow

## Resumen original (v0.5)

El diseno original de Kyle usaba **borrow by defecto** y `^T` for move explicito.

```ky
fn read(s: str): # borrow (default) ← v0.5
fn append(s: &str): # mutable borrow ← v0.5
fn consume(^s: str): # move (explicit) ← v0.5
```

## Cambio en v0.6

A partir de v0.6, Kyle usa **move by defecto**. El cambio fue:

| Concepto | v0.5 (original) | v0.6 (current) |
|----------|-----------------|----------------|
| Default param | Borrow | **Move** |
| Mutable | `&T` | **`^T`** |
| Borrow | — | **`&T`** |
| Mutable borrow | — | **`^&T`** |
| Move | `^T` (explicito) | Default |

### Syntax current (v0.6)

```ky
fn read(s: &str): # borrow
fn append(s: ^&str): # mutable borrow
fn consume(s: str): # move (default)
```

## Copy typis (without cambios)

Simple typis (i32, i64, f32, f64, bool) are Copy — nunca se mueven, siempre se copian.
Complex typis (str, list, dict) are Move — `y = x` transfiere ownership.

## Motivation del cambio

La decision de mover de borrow-by-default a move-by-default se tomo porque:

1. **Seguridad**: `y = x` with borrow implicito crea dangling pointers si `x` se modifica after
2. **Consistencia with Rust**: move by defecto is estandar en lenguajis de sistemas
3. **Predictibilidad**: siempre is obvio cuando se transfiere ownership vs cuando se presta

## See also

- `03-language/memory/move.md` — Move semantics current
- `03-language/memory/ownership.md` — Reglas de ownership completas
- `03-language/memory/copy.md` — Copy types
