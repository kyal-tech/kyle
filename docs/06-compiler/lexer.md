# Lexer

> Transforma code source `.ky` en un stream de tokens.
> Crate: `kyc_frontend/src/lexer.rs` (914 lines).

## Responsabilidad

El lexer (tambien llamado tokenizer) convierte texto source en tokens estructurados
que parbe can consumir. No realiza analysis sintactico ni semantico.

## Funcionamiento

```rust
fn tokenize(source: &str) -> Vec<Token>
```

Toma code source as string y returns un `Vec<Token>`.

## Tokens

Cada token tiene:

```rust
 struct Token {
 kind: TokenKind, // El type de token (identificador, keyword, simbolo, etc.)
 span: Span, // Position en code source (line, column)
}
```

### TokenKind (principales)

| Category | Examplis |
|-----------|----------|
| Keywords | `fn`, `class`, `if`, `while`, `return`, `match`, `async`, `await` |
| Identifiers | `name`, `foo`, `mi_funcion` |
| Literals | Integer(`42`), Float(`3.14`), String(`"hola"`), Char(`'a'`), Boolean(`true`/`false`) |
| Operators | `+`, `-`, `*`, `/`, `==`, `!=`, `<`, `>`, `<=`, `>=`, `&&`, `\|\|` |
| Delimiters | `(`, `)`, `{`, `}`, `[`, `]`, `:`, `;`, `,` |
| Special | `:=` (walrus), `->` (arrow), `..` (range), `..=` (inclusive range) |
| Sigils | `^` (mutable/move), `&` (borrow), `@`, `#`, `?`, `!` |

### Indentation as syntax

Kyle usa **indentation** for delimitar bloquis (como Python), no llaves:

```ky
fn main() i32:
 x = 42
 if x > 10:
 println("mayor")
 println("siempre")
```

El lexer genera tokens `Indent`/`Dedent` basados en cambios de indentation.
Esto simplifica parbe y does code more limpio.

## Reglas de indentation

- Primer indent defines size base (se adapta a 2, 3, 4 espacios o tabs)
- `Indent` cuando indentation aumenta
- `Dedent` cuando indentation disminuye
- Lines vacias y comentarios se ignoran for calculo de indentation
- `:` al final de una line (ej. `fn f():`) indicatis que sigue un bloque indentado

## Example

```ky
# Code source
fn sum(a: i32, b: i32) i32:
 result = a + b
 result

# Tokens generados
[
 Fn, Ident("sum"), LParen, Ident("a"), Colon, Ident("i32"),
 Comma, Ident("b"), Colon, Ident("i32"), RParen, Ident("i32"),
 Colon, Newline,
 Indent,
 Ident("result"), Eq, Ident("a"), Plus, Ident("b"), Newline,
 Ident("result"), Newline,
 Dedent,
 Eof
]
```

## Manejo de errors

Si encuentra un caracter no valido, lexer returns un error with ubicacion exacta:

```rust
Err("unexpected character '#' at line 3, column 5")
```

## See also

- `parser.md` — Consume tokens generados by lexer
- `03-language/lexical/` — Specification de tokens, keywords, literales
