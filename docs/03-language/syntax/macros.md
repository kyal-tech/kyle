# Macros

> **No implemented.** Kyle no has sistema de macros currentmente.

## Status

| Type de macro | Status |
|---------------|--------|
| `#[attribute]` | ✅ Parcial (solo `#[test]`, `#[bench]`) |
| Macros proceduralis | ❌ No implemented |
| Macros declarativas (`macro_rules!`) | ❌ No implemented |
| `compile-time` functions | ❌ No implemented |

## Alternativas currentes

Para metaprogramacion, usa functions y genericos:

```ky
fn max<T: copy>(a: T, b: T) T:
 if a > b: a else: b

fn clamp<T: copy>(val: T, min: T, max: T) T:
 if val < min: min
 else if val > max: max
 else: val
```

## Futuro

Las macros are en roadmap for permitir:
- DSLs embebidos
- Generation de code
- Serialization automatica
- Traits derive

## See also

- `generics.md` — Generics (alternativa a macros)
- `attributes.md` — Attributis existentes
