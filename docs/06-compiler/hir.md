# HIR (High-Level Intermediate Representation)

> AST desugerado. Simplifica construccionis sintacticas before del analysis semantico.
> Crate: `kyc_hir/src/lib.rs` (422 lines).

## Responsabilidad

El HIR transforma AST en una forma more simple eliminando sugar sintactico.
Operacionis as `for-in-range` se convierten a `while`, operadoris compuestos
(`+=`) se expanden, y otras simplificacionis ocurren aqui.

## Desugaring

| Construction original | Transformation HIR |
|----------------------|-------------------|
| `for i in 0..10:` | `i = 0; while i < 10: ... i += 1` |
| `x += expr` | `x = x + expr` |
| `x ?? default` | `if x is Some(v): v else: default` |
| `T?` optional | `Option<T>` struct |
| `T!` fallible | `Result<T, str>` struct |
| Tuple destructuring | Variable assignments |
| String interpolation | String concatenation |

## Pipeline

```
AST
 │ desugar_expr() / desugar_type()
 ▼
HIR (same AST enum, simplified)
```

El HIR usa **mismos types** que AST (`Expr`, `Stmt`, etc.) pero con
construccionis simplificadas. No introduce un nuevo type de nodo.

## Implementation

```rust
fn desugar_module(module: &Module) -> Module {
 Module {
 declarations: module.declarations.iter().map(desugar_decl).collect(),
 statements: module.statements.iter().map(desugar_stmt).collect(),
 ..module.clone()
 }
}

fn desugar_expr(expr: &Expr) -> Expr {
 match expr {
 Expr::Binary { left, operator: BinaryOp::AddAssign, right, span } => {
 // x += y → x = x + y
 Expr::Assignment {
 target: left.clone(),
 value: Box::new(Expr::Binary {
 left: left.clone(),
 operator: BinaryOp::Add,
 right: right.clone(),
 span: span.clone(),
 }),
 operator: None,
 span: span.clone(),
 }
 }
 Expr::For { iterable, body, .. } => {
 // for i in range: → while loop
 desugar_for_to_while(iterable, body)
 }
 // ... otros cases
 }
}
```

## See also

- `parser.md` — Genera AST input del HIR
- `semantic.md` — Consume HIR for type checking
