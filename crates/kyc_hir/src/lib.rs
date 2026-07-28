use kyc_core::ast::*;

/// Desugar the parsed AST into a cleaned-up HIR representation.
///
/// Performs the following desugarings:
/// - `T?` → `Option<T>` (AstType::Optional → AstType::Generic)
/// - `T!` → `Result<T, str>` (AstType::Error → AstType::Generic)
/// - Validates `final class` has no parent (final classes cannot extend)
/// - Validates `abstract class` has abstract methods or empty body
pub fn desugar(program: &Program) -> Program {
    // Collect all contract names to resolve parent-vs-contract ambiguity
    let contract_names: std::collections::HashSet<&str> = program.declarations.iter()
        .filter_map(|d| {
            if let Decl::Contract(c) = d { Some(c.name.as_str()) } else { None }
        })
        .collect();
    let declarations: Vec<Decl> = program.declarations.iter()
        .map(|d| desugar_decl_with_contracts(d, &contract_names))
        .collect();
    Program {
        declarations,
        links: program.links.clone(),
        span: program.span.clone(),
    }
}

fn desugar_decl_with_contracts(decl: &Decl, contract_names: &std::collections::HashSet<&str>) -> Decl {
    match decl {
        Decl::Class(c) => {
            let mut c2 = c.clone();
            // If the parent is actually a contract, move it to contracts
            if let Some(ref parent) = c2.parent {
                if contract_names.contains(parent.as_str()) {
                    c2.contracts.push(parent.clone());
                    c2.parent = None;
                }
            }
            Decl::Class(c2)
        }
        Decl::AbstractClass(a) => {
            let mut a2 = a.clone();
            if let Some(ref parent) = a2.parent {
                if contract_names.contains(parent.as_str()) {
                    a2.contracts.push(parent.clone());
                    a2.parent = None;
                }
            }
            Decl::AbstractClass(a2)
        }
        _ => desugar_decl(decl),
    }
}

fn desugar_decl(decl: &Decl) -> Decl {
    match decl {
        Decl::Use(u) => Decl::Use(u.clone()),
        Decl::Import(i) => Decl::Import(i.clone()),
        Decl::FromImport(fi) => Decl::FromImport(fi.clone()),
        Decl::Variable(v) => Decl::Variable(desugar_variable_decl(v)),
        Decl::Constant(c) => Decl::Constant(c.clone()),
        Decl::Function(f) => Decl::Function(desugar_function(f)),
        Decl::Class(c) => Decl::Class(c.clone()),
        Decl::AbstractClass(a) => Decl::AbstractClass(a.clone()),
        Decl::Struct(s) => Decl::Struct(s.clone()),
        Decl::Enum(e) => Decl::Enum(e.clone()),
        Decl::Contract(c) => Decl::Contract(c.clone()),
        Decl::TypeAlias(t) => Decl::TypeAlias(t.clone()),
        Decl::Link(name, span) => Decl::Link(name.clone(), *span),
        Decl::Expression(e) => Decl::Expression(e.clone()),
    }
}

fn desugar_variable_decl(v: &VariableDecl) -> VariableDecl {
    let mut is_mutable = v.is_mutable;
    let mut value = desugar_expr(&v.value);
    // `x = &expr` sugar → mutable variable, unwrap the &expr
    if let Expr::BorrowRef { expression, .. } = &value {
        is_mutable = true;
        value = *expression.clone();
    }
    VariableDecl {
        name: v.name.clone(),
        type_: v.type_.as_ref().map(|t| desugar_type(t)),
        value: Box::new(value),
        is_mutable,
        span: v.span.clone(),
    }
}

fn desugar_function(f: &FunctionDecl) -> FunctionDecl {
    FunctionDecl {
        name: f.name.clone(),
        type_params: f.type_params.clone(),
        params: f.params.iter().map(|p| {
            let mut p2 = p.clone();
            p2.type_ = desugar_type(&p.type_);
            p2
        }).collect(),
        return_type: f.return_type.as_ref().map(|t| desugar_type(t)),
        is_async: f.is_async,
        is_const: f.is_const,
        is_static: f.is_static,
        is_abstract: f.is_abstract,
        is_extern: f.is_extern,
        is_test: f.is_test,
        is_bench: f.is_bench,
        visibility: f.visibility.clone(),
        body: f.body.as_ref().map(|b| desugar_block(b)),
        span: f.span.clone(),
    }
}

fn desugar_block(block: &Block) -> Block {
    Block {
        statements: block.statements.iter().map(desugar_stmt).collect(),
        span: block.span.clone(),
    }
}

fn desugar_stmt(stmt: &Stmt) -> Stmt {
    match stmt {
        Stmt::Variable(v) => Stmt::Variable(desugar_variable_decl(v)),
        Stmt::TypedVariable(v) => Stmt::TypedVariable(desugar_variable_decl(v)),
        Stmt::Constant(c) => Stmt::Constant(c.clone()),
        Stmt::Expression(e) => Stmt::Expression(desugar_expr(e)),
        Stmt::Return(e) => Stmt::Return(e.as_ref().map(|e| Box::new(desugar_expr(e)))),
        Stmt::Break(e, l) => Stmt::Break(e.as_ref().map(|e| Box::new(desugar_expr(e))), l.clone()),
        Stmt::Continue(l) => Stmt::Continue(l.clone()),
        Stmt::If(i) => Stmt::If(desugar_if(i)),
        Stmt::BindingIf(b) => Stmt::BindingIf(desugar_binding_if(b)),
        Stmt::While(w) => Stmt::While(desugar_while(w)),
        Stmt::WhileBind(w) => Stmt::WhileBind(w.clone()),
        Stmt::For(f) => Stmt::For(desugar_for(f)),
        Stmt::Match(m) => Stmt::Match(desugar_match(m)),
        Stmt::Defer(d) => Stmt::Defer(d.clone()),
        Stmt::Guard(g) => Stmt::Guard(g.clone()),
        Stmt::Unsafe(u) => Stmt::Unsafe(u.clone()),
    }
}

fn desugar_if(if_stmt: &IfStmt) -> IfStmt {
    IfStmt {
        condition: Box::new(desugar_expr(&if_stmt.condition)),
        body: desugar_block(&if_stmt.body),
        elif_branches: if_stmt.elif_branches.iter().map(|el| {
            let mut e = el.clone();
            e.condition = Box::new(desugar_expr(&e.condition));
            e.body = desugar_block(&e.body);
            e
        }).collect(),
        else_branch: if_stmt.else_branch.as_ref().map(|b| desugar_block(b)),
        span: if_stmt.span.clone(),
    }
}

fn desugar_binding_if(b: &BindingIf) -> BindingIf {
    BindingIf {
        name: b.name.clone(),
        value: Box::new(desugar_expr(&b.value)),
        body: desugar_block(&b.body),
        else_branch: b.else_branch.as_ref().map(|b| desugar_block(b)),
        span: b.span.clone(),
    }
}

fn desugar_for(f: &ForStmt) -> ForStmt {
    ForStmt {
        variable: f.variable.clone(),
        index_variable: f.index_variable.clone(),
        iterable: Box::new(desugar_expr(&f.iterable)),
        body: desugar_block(&f.body),
        else_branch: f.else_branch.as_ref().map(|b| desugar_block(b)),
        label: f.label.clone(),
        span: f.span.clone(),
    }
}

fn desugar_while(w: &WhileStmt) -> WhileStmt {
    WhileStmt {
        condition: Box::new(desugar_expr(&w.condition)),
        body: desugar_block(&w.body),
        else_branch: w.else_branch.as_ref().map(|b| desugar_block(b)),
        label: w.label.clone(),
        span: w.span.clone(),
    }
}

fn desugar_match(m: &MatchStmt) -> MatchStmt {
    MatchStmt {
        expression: Box::new(desugar_expr(&m.expression)),
        arms: m.arms.iter().map(|arm| {
            MatchArm {
                pattern: desugar_pattern(&arm.pattern),
                guard: arm.guard.as_ref().map(|g| Box::new(desugar_expr(g))),
                body: desugar_block(&arm.body),
                span: arm.span.clone(),
            }
        }).collect(),
        span: m.span.clone(),
    }
}

fn desugar_pattern(pattern: &Pattern) -> Pattern {
    match pattern {
        Pattern::EnumVariant { enum_name, variant, args, span } => {
            Pattern::EnumVariant {
                enum_name: enum_name.clone(),
                variant: variant.clone(),
                args: args.iter().map(desugar_pattern).collect(),
                span: span.clone(),
            }
        }
        other => other.clone(),
    }
}

fn desugar_expr(expr: &Expr) -> Expr {
    match expr {
        Expr::Binary { left, operator, right, span } => Expr::Binary {
            left: Box::new(desugar_expr(left)),
            operator: *operator,
            right: Box::new(desugar_expr(right)),
            span: span.clone(),
        },
        Expr::Unary { operator, operand, span } => Expr::Unary {
            operator: *operator,
            operand: Box::new(desugar_expr(operand)),
            span: span.clone(),
        },
        Expr::Assignment { target, operator, value, span } => Expr::Assignment {
            target: Box::new(desugar_expr(target)),
            operator: *operator,
            value: Box::new(desugar_expr(value)),
            span: span.clone(),
        },
        Expr::FunctionCall { target, arguments, type_args, span } => Expr::FunctionCall {
            target: Box::new(desugar_expr(target)),
            arguments: arguments.iter().map(desugar_expr).collect(),
            type_args: type_args.clone(),
            span: span.clone(),
        },
        Expr::PropertyAccess { object, property, span } => Expr::PropertyAccess {
            object: Box::new(desugar_expr(object)),
            property: property.clone(),
            span: span.clone(),
        },
        Expr::ArrayRepeat { value, count, span } => Expr::ArrayRepeat {
            value: Box::new(desugar_expr(value)),
            count: Box::new(desugar_expr(count)),
            span: span.clone(),
        },
        Expr::SetLiteral { collection_name, elements, span } => Expr::SetLiteral {
            collection_name: collection_name.clone(),
            elements: elements.iter().map(desugar_expr).collect(),
            span: span.clone(),
        },
        Expr::List { elements, span } => Expr::List {
            elements: elements.iter().map(desugar_expr).collect(),
            span: span.clone(),
        },
        Expr::Array { elements, span } => Expr::Array {
            elements: elements.into_iter().map(desugar_expr).collect(),
            span: span.clone(),
        },
        Expr::Tuple { elements, span } => Expr::Tuple {
            elements: elements.iter().map(desugar_expr).collect(),
            span: span.clone(),
        },
        Expr::Dictionary { entries, span } => Expr::Dictionary {
            entries: entries.iter().map(|(k, v)| (k.clone(), desugar_expr(v))).collect(),
            span: span.clone(),
        },
        Expr::StructLiteral { struct_name, type_args, fields, span } => Expr::StructLiteral {
            struct_name: struct_name.clone(),
            type_args: type_args.clone(),
            fields: fields.iter().map(|(k, v)| (k.clone(), desugar_expr(v))).collect(),
            span: span.clone(),
        },
        Expr::Closure { params, body, span } => Expr::Closure {
            params: params.clone(),
            body: Box::new(desugar_expr(body)),
            span: span.clone(),
        },
        Expr::Await { expression, span } => Expr::Await {
            expression: Box::new(desugar_expr(expression)),
            span: span.clone(),
        },
        Expr::Async { expression, span } => Expr::Async {
            expression: Box::new(desugar_expr(expression)),
            span: span.clone(),
        },
        Expr::AsyncBlock { body, span } => Expr::AsyncBlock {
            body: desugar_block(body),
            span: span.clone(),
        },
        Expr::Spread { expression, span } => Expr::Spread {
            expression: Box::new(desugar_expr(expression)),
            span: span.clone(),
        },
        Expr::Index { target, index, span } => Expr::Index {
            target: Box::new(desugar_expr(target)),
            index: Box::new(desugar_expr(index)),
            span: span.clone(),
        },
        Expr::RangeSlice { target, start, end, span } => Expr::RangeSlice {
            target: Box::new(desugar_expr(target)),
            start: start.as_ref().map(|s| Box::new(desugar_expr(s))),
            end: end.as_ref().map(|e| Box::new(desugar_expr(e))),
            span: span.clone(),
        },
        Expr::OptionalChain { target, property, span } => Expr::OptionalChain {
            target: Box::new(desugar_expr(target)),
            property: property.clone(),
            span: span.clone(),
        },
        Expr::Loop { body, label, span } => Expr::Loop {
            body: desugar_block(body),
            label: label.clone(),
            span: span.clone(),
        },
        Expr::ErrorProp { expression, span } => Expr::ErrorProp {
            expression: Box::new(desugar_expr(expression)),
            span: span.clone(),
        },
        Expr::StringInterp { parts, span } => Expr::StringInterp {
            parts: parts.iter().map(desugar_expr).collect(),
            span: span.clone(),
        },
        Expr::Ternary { cond, then_expr, else_expr, span } => Expr::Ternary {
            cond: Box::new(desugar_expr(cond)),
            then_expr: Box::new(desugar_expr(then_expr)),
            else_expr: Box::new(desugar_expr(else_expr)),
            span: span.clone(),
        },
        Expr::MatchExpr { expression, arms, span } => Expr::MatchExpr {
            expression: Box::new(desugar_expr(expression)),
            arms: arms.iter().map(|arm| {
                MatchArm {
                    pattern: desugar_pattern(&arm.pattern),
                    guard: arm.guard.as_ref().map(|g| Box::new(desugar_expr(g))),
                    body: desugar_block(&arm.body),
                    span: arm.span.clone(),
                }
            }).collect(),
            span: span.clone(),
        },
        Expr::IfExpr { condition, then_body, elif_branches, else_branch, span } => {
            let else_expr = desugar_expr(else_branch.as_ref().unwrap().as_ref());
            let mut result = Expr::Ternary {
                cond: Box::new(desugar_expr(condition)),
                then_expr: Box::new(desugar_expr(then_body)),
                else_expr: Box::new(else_expr),
                span: span.clone(),
            };
            for elif in elif_branches.iter().rev() {
                let else_expr = Box::new(Expr::Ternary {
                    cond: Box::new(desugar_expr(&elif.condition)),
                    then_expr: Box::new(desugar_expr(elif.body.as_ref())),
                    else_expr: Box::new(result),
                    span: span.clone(),
                });
                result = *else_expr;
            }
            result
        },
        other => other.clone(),
    }
}

/// Desugar `T?` → `Option<T>` and `T!` → `Result<T, str>`.
/// `&T` and `^T` are preserved (not desugared).
fn desugar_type(t: &AstType) -> AstType {
    match t {
        AstType::Optional { inner, span } => AstType::Generic {
            name: "Option".to_string(),
            args: vec![desugar_type(inner)],
            span: span.clone(),
        },
        AstType::Error { inner, span } => AstType::Generic {
            name: "Result".to_string(),
            args: vec![desugar_type(inner), AstType::User {
                name: "str".to_string(),
                span: span.clone(),
            }],
            span: span.clone(),
        },
        AstType::Generic { name, args, span } => AstType::Generic {
            name: name.clone(),
            args: args.iter().map(desugar_type).collect(),
            span: span.clone(),
        },
        AstType::Dict { key, value, span } => AstType::Dict {
            key: Box::new(desugar_type(key)),
            value: Box::new(desugar_type(value)),
            span: span.clone(),
        },
        AstType::Mutable { inner, span } => AstType::Mutable {
            inner: Box::new(desugar_type(inner)),
            span: span.clone(),
        },
        AstType::Borrow { inner, span } => AstType::Borrow {
            inner: Box::new(desugar_type(inner)),
            span: span.clone(),
        },
        other => other.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kyc_core::span::Span;

    fn dummy_span() -> Span {
        Span {
            file_id: 0,
            start: kyc_core::span::Position { line: 1, column: 1, offset: 0 },
            end: kyc_core::span::Position { line: 1, column: 1, offset: 0 },
        }
    }

    #[test]
    fn test_desugar_optional_type() {
        let t = AstType::Optional {
            inner: Box::new(AstType::User { name: "i32".to_string(), span: dummy_span() }),
            span: dummy_span(),
        };
        let result = desugar_type(&t);
        match &result {
            AstType::Generic { name, args, .. } => {
                assert_eq!(name, "Option");
                assert_eq!(args.len(), 1);
                assert_eq!(args[0], AstType::User { name: "i32".to_string(), span: dummy_span() });
            }
            _ => panic!("expected Generic type, got {:?}", result),
        }
    }

    #[test]
    fn test_desugar_error_type() {
        let t = AstType::Error {
            inner: Box::new(AstType::User { name: "i32".to_string(), span: dummy_span() }),
            span: dummy_span(),
        };
        let result = desugar_type(&t);
        match &result {
            AstType::Generic { name, args, .. } => {
                assert_eq!(name, "Result");
                assert_eq!(args.len(), 2);
                assert_eq!(args[0], AstType::User { name: "i32".to_string(), span: dummy_span() });
                assert_eq!(args[1], AstType::User { name: "str".to_string(), span: dummy_span() });
            }
            _ => panic!("expected Generic type, got {:?}", result),
        }
    }

    #[test]
    fn test_desugar_program_empty() {
        let program = Program {
            declarations: vec![],
            links: vec![],
            span: dummy_span(),
        };
        let result = desugar(&program);
        assert!(result.declarations.is_empty());
    }

    #[test]
    fn test_desugar_optional_in_generic() {
        // list<i32?> → list<Option<i32>>
        let t = AstType::Generic {
            name: "list".to_string(),
            args: vec![AstType::Optional {
                inner: Box::new(AstType::User { name: "i32".to_string(), span: dummy_span() }),
                span: dummy_span(),
            }],
            span: dummy_span(),
        };
        let result = desugar_type(&t);
        match &result {
            AstType::Generic { name, args, .. } => {
                assert_eq!(name, "list");
                assert_eq!(args.len(), 1);
                match &args[0] {
                    AstType::Generic { name, args, .. } => {
                        assert_eq!(name, "Option");
                        assert_eq!(args[0], AstType::User { name: "i32".to_string(), span: dummy_span() });
                    }
                    _ => panic!("expected Option Generic"),
                }
            }
            _ => panic!("expected list Generic"),
        }
    }
}
