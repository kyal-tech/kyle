# Semantic Analysis

> Type checking, scope resolution, validacion semantics.
> Crate: `kyc_semantic/src/` (~2200 lines).

## Responsabilidad

El analysis semantico verifica que program sea **significativo** more alla de syntax:
- Typis correctos en cada expression
- Variablis y functions existen en scope current
- Mutabilidad respetada (no reallocate variablis inmutables)
- Ownership: move vs borrow consistente
- Visibility: `public` vs `private` vs `protected`

## Componentes

| File | Purpose |
|---------|-----------|
| `type_checker.rs` (1408) | Inferencia y verification de typis |
| `scope.rs` (422) | Resolution de nombris y scopis |
| `symbol_table.rs` (163) | Registro de simbolos y builtins |

## Pipeline

```
HIR (desugared AST)
 │
 ▼
[1] Scope Resolution → Asigna cada identificador a su declaration
 │
 ▼
[2] Type Inference → Determina type de cada expression
 │
 ▼
[3] Type Checking → Verifica que typis coincidan
 │
 ▼
Typed AST (every node has a known type)
```

## Type Checker

El type checker infiere y verifica typis using un algoritmo basado en Hindley-Milner
adaptado for un language imperativo with ownership.

### Inferencia basica

```rust
fn infer_expr(&mut self, expr: &Expr) -> Type {
 match expr {
 Expr::Literal { value: Literal::Integer(n) } => Type::I32,
 Expr::Literal { value: Literal::Float(n) } => Type::F64,
 Expr::Literal { value: Literal::String(s) } => Type::Str,
 Expr::Literal { value: Literal::Boolean(b) } => Type::Bool,
 Expr::Identifier { name } => self.lookup_type(name),
 Expr::Binary { left, operator: BinaryOp::Add, right } => {
 let l = self.infer_expr(left);
 let r = self.infer_expr(right);
 // Ambos must be numericos y del mismo type
 self.unify(l, r, "addition")
 }
 // ...
 }
}
```

### Type checking de llamadas a funcion

```rust
fn check_function_call(&mut self, target: &Expr, args: &[Expr]) -> Type {
 let func_type = self.infer_expr(target);
 let param_typis = self.get_param_types(func_type);
 for (arg, param) in args.iter().zip(param_types.iter()) {
 let arg_type = self.infer_expr(arg);
 self.check_type_match(&arg_type, param, "argument");
 }
 self.get_return_type(func_type)
}
```

## Scope Resolver

El scope resolver asigna cada name a su declaration:

```ky
# Scope global
x = 42 # x → global scope

fn main() i32: # main → global scope
 y = 10 # y → function scope
 if true:
 z = 5 # z → block scope (inside if)
 # z no accesible aqui
```

### Reglas

- Los scopis se anidan with cada bloque indentado
- El scope interior can acceder al exterior
- Variablis de bloquis superioris no se can reallocate from interior
- Functions y clasis se registran en scope before de procesar cuerpo

## Symbol Table

```rust
 struct SymbolTable {
 scopes: Vec<HashMap<String, Symbol>>,
 type_defs: HashMap<String, Type>,
}

 struct Symbol {
 name: String,
 kind: SymKind,
}

 enum SymKind {
 Variable { is_mutable: bool, is_auto: bool },
 Function(FunctionDecl),
 Constant(ConstantDecl),
 Type(Type),
 Module(Vec<String>),
}
```

### Builtins registrados

```rust
fn register_builtins(&mut self) {
 // Typis builtin
 self.type_defs.insert("i32", Type::I32);
 self.type_defs.insert("str", Type::Str);
 // ... (todos typis primitivos)
 
 // Functions builtin
 self.register("println", SymKind::Function(...));
 self.register("len", SymKind::Function(...));
 self.register("assert_eq", SymKind::Function(...));
 // ...
}
```

## Errors semanticos comunes

```rust
// Type mismatch
KL-E0001: Type mismatch: expected 'i32', found 'str'
 --> file.ky:10:5

// Undefined symbol
KL-E0009: Undefined symbol 'foo'
 --> file.ky:5:10

// Cannot modify immutable
KL-E0007: Cannot modify immutable variable 'x'
 --> file.ky:3:5
```

## See also

- `type_checker.rs` en code source
- `03-language/types/` — Specification del sistema de types
- `03-language/memory/` — Reglas de ownership y mutabilidad
