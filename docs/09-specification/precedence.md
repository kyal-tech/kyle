# Operator Precedence

> Tabla de precedencia de operadoris en Kyle.
> De mayor a menor precedencia.

| Nivel | Operadoris | Description |
|-------|-----------|-------------|
| 17 | `()` `[]` `.` `::` | Llamada, index, acceso, herencia |
| 16 | `as` `is` `not` | Cast, type check, negation |
| 15 | `^` `&` `*` `-` (unario) | Sigilos y negation unaria |
| 14 | `*` `/` `%` | Multiplication, division, module |
| 13 | `+` `-` | Suma, resta |
| 12 | `<<` `>>` | Shift bitwise |
| 11 | `&` `\|` `^` (bitwise) | Bitwise AND, OR, XOR |
| 10 | `..` `..=` | Range (inclusive/exclusive) |
| 9 | `==` `!=` `<` `>` `<=` `>=` | Comparison |
| 8 | `and` | AND logico |
| 7 | `or` | OR logico |
| 6 | `??` | Null-coalescing |
| 5 | `..` (range) | Range en for |
| 4 | `=` `+=` `-=` etc. | Assignment |
| 3 | `->` | Arrow (return type) |
| 2 | `:` | Type annotation |
| 1 | `:=` | Constante |

## Notes

- Los operadoris del mismo nivel se evaluan de izquierda a derecha
- Usar parentesis for desambiguar: `(a + b) * c`
- `not` has mayor precedencia que `and`/`or`
- `??` is asociativo a derecha: `a ?? b ?? c` = `a ?? (b ?? c)`

## See also

- `03-language/lexical/operators.md` — List completa de operadores
- `03-language/syntax/expressions.md` — Expresionis del language
