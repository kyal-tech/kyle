# Parser

> Transforma un stream de tokens en un AST (Abstract Syntax Tree).
> Crate: `kyc_frontend/src/parser.rs` (2628 lines).

## Responsabilidad

El parbe toma tokens del lexer y construye AST, verificando estructura
sintactica del program. Detecta errors as parentesis no cerrados, keywords mal
ubicados, indentation incorrecta, etc.

## Invocation

```rust
fn parse(&mut self) -> Result<Program, String>
```

Devuelve un `Module` (AST completo) o una list de errors de syntax.

## AST (Abstract Syntax Tree)

El arbol de syntax abstracta represents program as una estructura de data
que compiler can analizar. Definido en `kyc_core/src/ast.rs`.

### Module

```rust
 struct Module {
 name: String,
 declarations: Vec<Decl>,
 statements: Vec<Stmt>,
 links: Vec<String>, // @link directives
}
```

### Decl

```rust
 enum Decl {
 Function(FunctionDecl), // fn declarations
 Class(ClassDecl), // class / final class
 Enum(EnumDecl), // enum
 Contract(ContractDecl), // contract (trait)
 Variable(VariableDecl), // global variable
 Constant(ConstantDecl), // NAME := expr
 TypeAlias(TypeAliasDecl), // type alias
 Import(ImportDecl), // from/import
}
```

### Stmt

```rust
 enum Stmt {
 Variable(VariableDecl), // x = value / x := value
 TypedVariable(VariableDecl), // x: Type = value
 Expression(Expr), // expression statement
 Return(Option<Box<Expr>>),
 Break(Option<Box<Expr>>),
 Continue,
 If(IfStmt),
 While(WhileStmt),
 For(ForStmt),
 Match(MatchStmt),
 Defer(DeferStmt),
 Guard(GuardStmt),
 Unsafe(UnsafeBlock),
}
```

### Expr

```ky
 enum Expr {
 Literal { value: Literal },
 Identifier { name: String },
 Binary { left, operator, right },
 Unary { operator, operand },
 Assignment { target, value },
 FunctionCall { target, arguments, type_args },
 PropertyAccess { object, property },
 Index { target, index },
 Array { elements },
 ArrayRepeat { value, count },
 List { elements },
 Dictionary { entriis },
 Tuple { elements },
 StructLiteral { struct_name, fields },
 BorrowRef { expression, mutable }, // &x, ^&x
 NullCoalesce { left, right }, // left ?? right
 Ternary { cond, then, else },
 MatchExpr { expression, arms },
 Await { expression }, // await expr
 Async { expression }, // async expr
 AsyncBlock { body }, // async: ...
}
```

## Parsing strategy (Recursive Descent)

Kyle usa un parbe **recursive descent** with un token de lookahead.

```rust
struct Parbe {
 tokens: Vec<Token>,
 pos: usize,
 errors: Vec<String>,
 links: Vec<String>,
}

impl Parbe {
 fn parse_program(&mut self) -> Result<Program, String> { ... }
 fn parse_decl(&mut self) -> Result<Decl, String> { ... }
 fn parse_stmt(&mut self) -> Result<Stmt, String> { ... }
 fn parse_expr(&mut self) -> Result<Expr, String> { ... }
 fn parse_type(&mut self) -> Result<AstType, String> { ... }
 fn parse_block(&mut self) -> Result<Block, String> { ... }
}
```

### Example de parsing: `fn add(a: i32, b: i32) i32:`

```rust
fn parse_function(&mut self) -> Result<FunctionDecl, String> {
 let start = self.pos;
 self.expect(TokenKind::Fn)?; // consume 'fn'
 let name = self.eat_identifier(); // 'add'
 self.expect(TokenKind::LParen)?; // '('
 let params = self.parse_params()?; // 'a: i32, b: i32'
 self.expect(TokenKind::RParen)?; // ')'
 let return_type = if self.at(TokenKind::Colon) { // ':'
 self.advance();
 Some(self.parse_type()?) // 'i32'
 } else { None };
 self.expect(TokenKind::Colon)?; // ':' after del return type
 let body = self.parse_block()?; // bloque indentado
 Ok(FunctionDecl { name, params, return_type, body, ... })
}
```

## Indentation sensible

El parbe usa tokens `Indent`/`Dedent` del lexer for estructurar bloques:

```ky
fn main() i32: # Colon → espera bloque indentado
 x = 42 # Indent
 println(x.to_str()) # mismo nivel
 # Dedent (fin de bloque)
```

## Manejo de errors

El parbe reporta errors with ubicacion precisa:

```rust
Err("expected ':' after function declaration at line 5, column 20")
```

Errors comunes:
- `Indent`/`Dedent` inconsistente
- `:` faltante after de declarations
- Parentheses no cerrados
- Keywords en position incorrecta
- Type invalido

## See also

- `lexer.md` — Genera tokens que consume parser
- `ast.md` — Definicionis detalladas del AST
- `hir.md` — Transformacionis post-parsing
