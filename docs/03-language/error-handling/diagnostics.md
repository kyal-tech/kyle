# Diagnostics

> Sistema de errors y diagnosticos del compiler Kyle.
> Reporta errors with code, mensaje y ubicacion precisa.

## Formato de error

```
KL-E0001: Type mismatch

 expected 'i32', found 'str'

 --> file.ky:10:5
 |
10 | x = "hello" + 42
 | ^^^^^^^^^^^^^ expected i32 here
 |
```

## Codis de error

| Code | Significado |
|--------|-------------|
| `KL-E0001` | Type mismatch |
| `KL-E0002` | Syntax error |
| `KL-E0003` | Undefined type |
| `KL-E0004` | Cannot modify immutable |
| `KL-E0005` | Undefined function |
| `KL-E0006` | Wrong number of arguments |
| `KL-E0007` | Cannot modify constant |
| `KL-E0008` | Invalid assignment target |
| `KL-E0009` | Undefined symbol |
| `KL-E0010` | Wrong number of type arguments |
| `KL-E0011` | Ambiguous type |
| `KL-E0012` | Return type mismatch |
| `KL-E0013` | Move analysis error |

## Warnings

| Warning | Condition |
|---------|-----------|
| Variable no usada | Declarada pero nunca leida |
| Import no usado | Module importado no utilizado |
| Code muerto | Code after de `return`/`break` |
| Conversion implicita | Sin `as` cast explicito |

## Move analysis errors

El borrow checker produce errors especificos:

```
KL-E0013: use-after-move: cannot move `s` (local #2) — value has been moved
KL-E0013: cannot mutably borrow `s` — it is already borrowed immutably
```

## See also

- `06-compiler/diagnostics.md` — Implementation del sistema de diagnosticos
- `06-compiler/borrow-analysis.md` — Errors del borrow checker
