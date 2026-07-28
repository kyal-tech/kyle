# AST (Abstract Syntax Tree)

> Representation estructurada del program after del parsing.
> Crate: `kyc_core/src/ast.rs` (~1372 lines).

## Responsabilidad

El AST is representscion intermedia between parbe y resto del compiler.
Es un arbol que refleja directamente syntax del program. A diferencia del HIR,
 AST rehas estructura sintactica exacta (incluyendo sugar sintactico).

## Estructura principal

```
Module
├── Declarations
│ ├── Function → fn declarations
│ ├── Class → class / final class
│ ├── Enum → enum variants
│ ├── Contract → contract (traits)
│ ├── Variable → global variables
│ ├── Constant → NAME := expr
│ ├── TypeAlias → type alias
│ └── Import → from/import
└── Statements (module-level)
 ├── Variable
 ├── Expression
 ├── If / While / For / Match
 └── Return / Break / Continue
```

## Typis (AstType)

```rust
 enum AstType {
 Primitive { name: String, span: Span },
 Ube { name: String, span: Span },
 Generic { name: String, args: Vec<AstType>, span: Span },
 Optional { inner: Box<AstType>, span: Span }, // T?
 Error { inner: Box<AstType>, span: Span }, // T!
 Dict { key: Box<AstType>, value: Box<AstType>, span: Span }, // {K: V}
 FnPtr { params: Vec<AstType>, return_: Box<AstType>, span: Span },
 Mutable { inner: Box<AstType>, span: Span }, // ^T
 Borrow { inner: Box<AstType>, span: Span }, // &T
 Array { inner: Box<AstType>, size: usize, span: Span }, // [T, N]
 Ptr { span: Span },
}
```

> **Note:** `{T}` (list) y `{K:V}` (dict) NO are `AstType` variants propios.
> Se representsn as `Generic { name: "list" }` / `Generic { name: "dict" }` en AST.
> Los structs literalis tamlittle have variant propia — se resuelven as typis usuario.

## Span

Cada nodo del AST has un `Span` que indicatis su position exacta en code source:

```rust
 struct Span {
 start: usize, // byte offset
 end: usize, // byte offset
 line: u32,
 column: u32,
}
```

Los spans are fundamentalis for reportar errors with ubicacion precisa.

## Example

```ky
# Code source
fn add(a: i32, b: i32) i32:
 a + b
```

Produce un AST como:

```
Module
└── Function "add"
 ├── Params
 │ ├── "a": Primitive("i32")
 │ └── "b": Primitive("i32")
 ├── Return: Primitive("i32")
 └── Body
 └── BinaryOp(Add)
 ├── Identifier("a")
 └── Identifier("b")
```

## See also

- `parser.md` — Construye AST
- `hir.md` — Desugaring del AST a HIR
