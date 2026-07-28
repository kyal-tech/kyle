# Diagnostics

> Sistema de errors y diagnosticos del compiler.
> Crate: `kyc_core/src/diagnostics.rs` / `kyc_core/src/span.rs`

## Responsabilidad

El sistema de diagnosticos provee errors y warnings with ubicacion precisa
en code source, facilitando depuracion.

## Error Codes

Cada error has un code unico:

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

## Estructura

```rust
 struct Diagnostic {
 code: ErrorCode,
 message: String,
 span: Option<Span>,
 notes: Vec<String>,
}

 struct Span {
 start: usize, // byte offset
 end: usize,
 line: u32,
 column: u32,
}
```

## Formato de output

```
KL-E0001: Type mismatch

 expected 'i32', found 'str'

 --> file.ky:10:5
 |
10 | x = "hello" + 42
 | ^^^^^^^^^^^^^ expected i32 here
 |
```

## Warnings

Ademore de errors, compiler produce warnings:

| Warning | Condition |
|---------|-----------|
| Unused variable | Variable declarada pero nunca usada |
| Unused import | Import de module no utilizado |
| Unreachable code | Code after de return/break |
| Deprecated feature | Uso de syntax obsoleta |
| Implicit conversion | Conversion de typis without `as` |

## See also

- `semantic.md` — Genera mayoria de errors semanticos
- `parser.md` — Genera errors sintacticos
- `borrow-analysis.md` — Genera errors de ownership
