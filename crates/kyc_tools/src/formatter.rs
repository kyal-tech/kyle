use std::fmt::Write;
use kyc_core::ast::*;
use crate::package::FormatConfig;

/// KL code formatter — parses source to AST and pretty-prints valid KL code.
pub struct Formatter {
    source_lines: Vec<String>,
    /// Current line index for comment tracking (1-indexed based on source lines).
    last_comment_line: usize,
    /// Formatting configuration.
    config: FormatConfig,
}

impl Formatter {
    pub fn new() -> Self {
        Self {
            source_lines: Vec::new(),
            last_comment_line: 0,
            config: FormatConfig::default(),
        }
    }

    pub fn with_config(config: FormatConfig) -> Self {
        Self {
            source_lines: Vec::new(),
            last_comment_line: 0,
            config,
        }
    }

    /// Format a KL source string.
    pub fn format(&self, source: &str) -> Result<String, String> {
        let mut lexer = kyc_frontend::lexer::Lexer::new(source);
        let tokens = lexer.tokenize();
        let mut parser = kyc_frontend::parser::Parser::new(tokens);
        let program = parser.parse()?;
        let mut f = Self {
            source_lines: source.lines().map(|l| l.to_string()).collect(),
            last_comment_line: 0,
            config: self.config.clone(),
        };
        Ok(f.format_program(&program))
    }

    /// Extract leading comment lines before a given source line (1-indexed).
    /// Returns comments with their indentation.
    /// Line is 1-indexed; source_lines is 0-indexed (line 1 → index 0).
    fn comments_before_line(&mut self, line: usize) -> Vec<(usize, String)> {
        let mut comments = Vec::new();
        let end = line.saturating_sub(1);
        if end <= self.last_comment_line {
            return comments;
        }
        for i in self.last_comment_line..end {
            if i >= self.source_lines.len() { break; }
            let trimmed = self.source_lines[i].trim();
            if trimmed.starts_with('#') {
                let indent = self.source_lines[i].len() - self.source_lines[i].trim_start().len();
                comments.push((indent, trimmed.to_string()));
            }
        }
        self.last_comment_line = line;
        comments
    }

    /// Collect all remaining comments from last_comment_line to end of file.
    fn collect_trailing_comments(&self) -> Vec<String> {
        let mut comments = Vec::new();
        let start = self.last_comment_line.saturating_sub(1);
        for i in start..self.source_lines.len() {
            let trimmed = self.source_lines[i].trim();
            if trimmed.starts_with('#') {
                comments.push(trimmed.to_string());
            }
        }
        comments
    }

    fn format_program(&mut self, program: &Program) -> String {
        let mut out = String::new();
        self.last_comment_line = 0;

        // Separate imports from other declarations
        let mut imports: Vec<&Decl> = Vec::new();
        let mut other: Vec<&Decl> = Vec::new();
        for decl in &program.declarations {
            match decl {
                Decl::Use(_) | Decl::Import(_) | Decl::FromImport(_) => imports.push(decl),
                _ => other.push(decl),
            }
        }

        // Sort imports: relative first, then absolute; alphabetically within each group
        imports.sort_by(|a, b| {
            let key_a = import_sort_key(a);
            let key_b = import_sort_key(b);
            key_a.cmp(&key_b)
        });

        // Write imports
        for decl in &imports {
            let span = decl_span(decl);
            let start_line = span.start.line as usize;
            for (_indent, comment) in self.comments_before_line(start_line) {
                out.push_str(&comment);
                out.push('\n');
            }
            self.write_decl(&mut out, decl, 0);
            out.push('\n');
        }
        if !imports.is_empty() && !other.is_empty() {
            out.push('\n');
        }

        // Write remaining declarations
        for decl in &other {
            let span = decl_span(decl);
            let start_line = span.start.line as usize;
            for (_indent, comment) in self.comments_before_line(start_line) {
                out.push_str(&comment);
                out.push('\n');
            }
            self.write_decl(&mut out, decl, 0);
            if !out.ends_with('\n') {
                out.push('\n');
            }
        }

        for comment in self.collect_trailing_comments() {
            out.push_str(&comment);
            out.push('\n');
        }
        if self.config.trailing_newline && !out.ends_with('\n') {
            out.push('\n');
        }
        out
    }

    fn indent(&mut self, out: &mut String, depth: usize) {
        let size = self.config.indent_size;
        for _ in 0..depth {
            for _ in 0..size {
                out.push(' ');
            }
        }
    }

    fn write_type(&mut self, out: &mut String, type_: &AstType) {
        match type_ {
            AstType::Primitive { name, .. } => out.push_str(name),
            AstType::User { name, .. } => out.push_str(name),
            AstType::Generic { name, args, .. } => {
                out.push_str(name);
                if !args.is_empty() {
                    out.push('<');
                    for (i, a) in args.iter().enumerate() {
                        if i > 0 { out.push_str(", "); }
                        self.write_type(out, a);
                    }
                    out.push('>');
                }
            }
            AstType::Optional { inner, .. } => {
                self.write_type(out, inner);
                out.push('?');
            }
            AstType::Error { inner, .. } => {
                self.write_type(out, inner);
                out.push('!');
            }
            AstType::Dict { key, value, .. } => {
                out.push_str("dict<");
                self.write_type(out, key);
                out.push_str(", ");
                self.write_type(out, value);
                out.push('>');
            }
            AstType::FnPtr { params, return_, .. } => {
                out.push('(');
                for (i, p) in params.iter().enumerate() {
                    if i > 0 { out.push_str(", "); }
                    self.write_type(out, p);
                }
                out.push_str(") ");
                self.write_type(out, return_);
            }
            AstType::Mutable { inner, .. } => {
                out.push('^');
                self.write_type(out, inner);
            }
            AstType::Borrow { inner, .. } => {
                out.push('&');
                self.write_type(out, inner);
            }
            AstType::Ptr { .. } => {
                out.push_str("ptr");
            }
            AstType::Set { inner, .. } => {
                out.push_str("set<");
                self.write_type(out, inner);
                out.push('>');
            }
            AstType::Queue { inner, .. } => {
                out.push_str("queue<");
                self.write_type(out, inner);
                out.push('>');
            }
            AstType::Stack { inner, .. } => {
                out.push_str("stack<");
                self.write_type(out, inner);
                out.push('>');
            }
            AstType::Deque { inner, .. } => {
                out.push_str("deque<");
                self.write_type(out, inner);
                out.push('>');
            }
            AstType::LinkedList { inner, .. } => {
                out.push_str("linked_list<");
                self.write_type(out, inner);
                out.push('>');
            }
            AstType::List { inner, .. } => {
                out.push('[');
                self.write_type(out, inner);
                out.push(']');
            }
            AstType::Array { inner, size, .. } => {
                out.push('[');
                self.write_type(out, inner);
                write!(out, "; {}", size).unwrap();
                out.push(']');
            }
            AstType::Slice { inner, .. } => {
                out.push_str("&[");
                self.write_type(out, inner);
                out.push(']');
            }
            AstType::Chan { inner, .. } => {
                out.push_str("chan<");
                self.write_type(out, inner);
                out.push('>');
            }
            AstType::Bytes { .. } => {
                out.push_str("bytes");
            }
        }
    }

    fn write_decl(&mut self, out: &mut String, decl: &Decl, depth: usize) {
        match decl {
            Decl::Use(u) => self.write_use(out, u, depth),
            Decl::Import(i) => self.write_import(out, i, depth),
            Decl::FromImport(fi) => self.write_from_import(out, fi, depth),
            Decl::Variable(v) => self.write_variable(out, v, depth),
            Decl::Constant(c) => self.write_constant(out, c, depth),
            Decl::Function(f) => self.write_function(out, f, depth),
            Decl::Class(c) => self.write_class(out, c, depth, false),
            Decl::AbstractClass(c) => self.write_class(out, &Self::abstract_to_class(c), depth, true),
            Decl::Struct(s) => self.write_struct(out, s, depth),
            Decl::Enum(e) => self.write_enum(out, e, depth),
            Decl::Contract(c) => self.write_contract(out, c, depth),
            Decl::TypeAlias(t) => self.write_type_alias(out, t, depth),
            Decl::Link(_, _) => {}
            Decl::Expression(_) => {}
        }
    }

    fn abstract_to_class(c: &AbstractClassDecl) -> ClassDecl {
        ClassDecl {
            name: c.name.clone(),
            type_params: c.type_params.clone(),
            parent: c.parent.clone(),
            contracts: c.contracts.clone(),
            members: c.members.clone(),
            span: c.span,
        }
    }

    fn write_use(&mut self, out: &mut String, u: &UseDecl, depth: usize) {
        self.indent(out, depth);
        write!(out, "use ").unwrap();
        if u.relative {
            write!(out, "~").unwrap();
        }
        write!(out, "{}", u.path.join(".")).unwrap();
        if let Some(names) = &u.names {
            write!(out, ".{{{}}}", names.join(", ")).unwrap();
        }
        if let Some(alias) = &u.alias {
            write!(out, " as {}", alias).unwrap();
        }
        writeln!(out).unwrap();
    }

    fn write_import(&mut self, out: &mut String, i: &Import, depth: usize) {
        self.indent(out, depth);
        if i.relative {
            write!(out, "import ~").unwrap();
        } else {
            write!(out, "import ").unwrap();
        }
        write!(out, "{}", i.module_name).unwrap();
        if let Some(alias) = &i.alias {
            write!(out, " as {}", alias).unwrap();
        }
    }

    fn write_from_import(&mut self, out: &mut String, fi: &FromImport, depth: usize) {
        self.indent(out, depth);
        if fi.relative {
            write!(out, "from ~{} ", fi.module_name).unwrap();
        } else {
            write!(out, "from {} ", fi.module_name).unwrap();
        }
        write!(out, "import {}", fi.imported_names.join(", ")).unwrap();
        if fi.imported_names.len() == 1 {
            if let Some(alias) = &fi.alias {
                write!(out, " as {}", alias).unwrap();
            }
        }
    }

    fn write_variable(&mut self, out: &mut String, v: &VariableDecl, depth: usize) {
        self.indent(out, depth);
        write!(out, "{}", v.name).unwrap();
        if let Some(type_) = &v.type_ {
            write!(out, ": ").unwrap();
            self.write_type(out, type_);
        }
        if v.is_mutable {
            write!(out, " := ").unwrap();
        } else {
            write!(out, " = ").unwrap();
        }
        self.write_expr(out, &v.value);
    }

    fn write_constant(&mut self, out: &mut String, c: &ConstantDecl, depth: usize) {
        self.indent(out, depth);
        write!(out, "{} := ", c.name).unwrap();
        self.write_expr(out, &c.value);
    }

    fn write_type_params(&mut self, out: &mut String, params: &[TypeParam]) {
        if params.is_empty() { return; }
        out.push('<');
        for (i, p) in params.iter().enumerate() {
            if i > 0 { out.push_str(", "); }
            out.push_str(&p.name);
            if let Some(constraint) = &p.constraint {
                out.push_str(": ");
                self.write_type(out, constraint);
            }
        }
        out.push('>');
    }

    fn write_params(&mut self, out: &mut String, params: &[Parameter]) {
        for (i, p) in params.iter().enumerate() {
            if i > 0 { out.push_str(", "); }
            out.push_str(&p.name);
            out.push_str(": ");
            self.write_type(out, &p.type_);
            if let Some(default) = &p.default {
                out.push_str(" = ");
                self.write_expr(out, default);
            }
        }
    }

    fn write_function(&mut self, out: &mut String, f: &FunctionDecl, depth: usize) {
        self.indent(out, depth);
        if f.is_test {
            write!(out, "#[test]\n").unwrap();
            self.indent(out, depth);
        }
        if f.is_bench {
            write!(out, "#[bench]\n").unwrap();
            self.indent(out, depth);
        }
        if f.is_const { write!(out, "const ").unwrap(); }
        if f.is_async { write!(out, "async ").unwrap(); }
        if f.is_abstract { write!(out, "abstract ").unwrap(); }
        write!(out, "fn {}", f.name).unwrap();
        self.write_type_params(out, &f.type_params);
        out.push('(');
        self.write_params(out, &f.params);
        out.push(')');
        if let Some(rt) = &f.return_type {
            write!(out, " ").unwrap();
            self.write_type(out, rt);
        }
        out.push_str(":\n");
        if let Some(body) = &f.body {
            self.write_block(out, body, depth + 1);
        }
    }

    fn write_class(&mut self, out: &mut String, c: &ClassDecl, depth: usize, is_abstract: bool) {
        self.indent(out, depth);
        if is_abstract {
            write!(out, "abstract class {}", c.name).unwrap();
        } else {
            write!(out, "final class {}", c.name).unwrap();
        }
        self.write_type_params(out, &c.type_params);
        if let Some(parent) = &c.parent {
            write!(out, " < {}", parent).unwrap();
        }
        if !c.contracts.is_empty() {
            write!(out, " implements {}", c.contracts.join(", ")).unwrap();
        }
        out.push_str(":\n");
        for member in &c.members {
            self.write_class_member(out, member, depth + 1);
        }
    }

    fn write_class_member(&mut self, out: &mut String, member: &ClassMember, depth: usize) {
        match member {
            ClassMember::Field(f) => {
                self.indent(out, depth);
                write!(out, "{}: ", f.name).unwrap();
                self.write_type(out, &f.type_);
                out.push('\n');
            }
            ClassMember::Property(p) => {
                self.indent(out, depth);
                write!(out, "{}: ", p.name).unwrap();
                self.write_type(out, &p.type_);
                if let Some(getter) = &p.getter {
                    write!(out, " get:\n").unwrap();
                    self.write_block(out, getter, depth + 1);
                }
                if let Some((_, setter)) = &p.setter {
                    write!(out, " set:\n").unwrap();
                    self.write_block(out, setter, depth + 1);
                }
            }
            ClassMember::Constructor(ctor) => {
                self.indent(out, depth);
                write!(out, "new(").unwrap();
                self.write_params(out, &ctor.params);
                out.push_str("):\n");
                self.write_block(out, &ctor.body, depth + 1);
            }
            ClassMember::Method(m) => self.write_function(out, m, depth),
        }
    }

    fn write_struct(&mut self, out: &mut String, s: &StructDecl, depth: usize) {
        self.indent(out, depth);
        write!(out, "struct {}", s.name).unwrap();
        self.write_type_params(out, &s.type_params);
        out.push_str(":\n");
        for field in &s.fields {
            self.indent(out, depth + 1);
            write!(out, "{}: ", field.name).unwrap();
            self.write_type(out, &field.type_);
            out.push('\n');
        }
    }

    fn write_enum(&mut self, out: &mut String, e: &EnumDecl, depth: usize) {
        self.indent(out, depth);
        write!(out, "enum {}", e.name).unwrap();
        self.write_type_params(out, &e.type_params);
        out.push_str(":\n");
        for variant in &e.variants {
            self.indent(out, depth + 1);
            write!(out, "{}", variant.name).unwrap();
            if !variant.payload.is_empty() {
                out.push('(');
                for (i, p) in variant.payload.iter().enumerate() {
                    if i > 0 { out.push_str(", "); }
                    self.write_type(out, p);
                }
                out.push(')');
            }
            out.push('\n');
        }
    }

    fn write_contract(&mut self, out: &mut String, c: &ContractDecl, depth: usize) {
        self.indent(out, depth);
        write!(out, "contract {}:\n", c.name).unwrap();
        for method in &c.methods {
            self.indent(out, depth + 1);
            write!(out, "fn {}(", method.name).unwrap();
            self.write_params(out, &method.params);
            out.push(')');
            if let Some(rt) = &method.return_type {
                write!(out, " ").unwrap();
                self.write_type(out, rt);
            }
            out.push('\n');
        }
    }

    fn write_type_alias(&mut self, out: &mut String, t: &TypeAlias, depth: usize) {
        self.indent(out, depth);
        write!(out, "type {}", t.name).unwrap();
        self.write_type_params(out, &t.type_params);
        write!(out, " = ").unwrap();
        self.write_type(out, &t.type_);
    }

    fn write_block(&mut self, out: &mut String, block: &Block, depth: usize) {
        for stmt in &block.statements {
            let span = stmt_span(stmt);
            let start_line = span.start.line as usize;
            if start_line > 0 {
                for (_indent, comment) in self.comments_before_line(start_line) {
                    for _ in 0..depth {
                        out.push_str("    ");
                    }
                    out.push_str(&comment);
                    out.push('\n');
                }
            }
            self.write_stmt(out, stmt, depth);
        }
    }

    fn write_stmt(&mut self, out: &mut String, stmt: &Stmt, depth: usize) {
        match stmt {
            Stmt::Expression(expr) => {
                self.indent(out, depth);
                self.write_expr(out, expr);
                out.push('\n');
            }
            Stmt::Variable(v) => self.write_variable(out, v, depth),
            Stmt::Return(Some(expr)) => {
                self.indent(out, depth);
                write!(out, "return ").unwrap();
                self.write_expr(out, expr);
                out.push('\n');
            }
            Stmt::Return(None) => {
                self.indent(out, depth);
                out.push_str("return\n");
            }
            Stmt::If(s) => self.write_if(out, s, depth),
            Stmt::While(s) => self.write_while(out, s, depth),
            Stmt::For(s) => self.write_for(out, s, depth),
            Stmt::Match(s) => self.write_match(out, s, depth),
            Stmt::Guard(g) => {
                self.indent(out, depth);
                out.push_str("guard:\n");
                self.write_block(out, &g.body, depth + 1);
            }
            Stmt::Unsafe(u) => {
                self.indent(out, depth);
                out.push_str("unsafe:\n");
                self.write_block(out, &u.body, depth + 1);
            }
            Stmt::Defer(d) => {
                self.indent(out, depth);
                write!(out, "defer ").unwrap();
                self.write_expr(out, &d.call);
                out.push('\n');
            }
            Stmt::BindingIf(b) => {
                self.indent(out, depth);
                write!(out, "if {} = ", b.name).unwrap();
                self.write_expr(out, &b.value);
                out.push_str(":\n");
                self.write_block(out, &b.body, depth + 1);
                if let Some(el) = &b.else_branch {
                    self.indent(out, depth);
                    out.push_str("else:\n");
                    self.write_block(out, el, depth + 1);
                }
            }
            Stmt::Break(_, _) => {
                self.indent(out, depth);
                out.push_str("break\n");
            }
            Stmt::Continue(_) => {
                self.indent(out, depth);
                out.push_str("continue\n");
            }
            Stmt::WhileBind(w) => {
                self.indent(out, depth);
                write!(out, "while {} = ", w.name).unwrap();
                self.write_expr(out, &w.iterable);
                out.push_str(":\n");
                self.write_block(out, &w.body, depth + 1);
            }
            Stmt::Constant(c) => self.write_constant(out, c, depth),
            Stmt::TypedVariable(v) => self.write_variable(out, &VariableDecl {
                name: v.name.clone(),
                type_: v.type_.clone(),
                value: v.value.clone(),
                is_mutable: false,
                span: v.span,
            }, depth),
        }
    }

    fn write_if(&mut self, out: &mut String, s: &IfStmt, depth: usize) {
        self.indent(out, depth);
        write!(out, "if ").unwrap();
        self.write_expr(out, &s.condition);
        out.push_str(":\n");
        self.write_block(out, &s.body, depth + 1);
        for elif in &s.elif_branches {
            self.indent(out, depth);
            write!(out, "elif ").unwrap();
            self.write_expr(out, &elif.condition);
            out.push_str(":\n");
            self.write_block(out, &elif.body, depth + 1);
        }
        if let Some(el) = &s.else_branch {
            self.indent(out, depth);
            out.push_str("else:\n");
            self.write_block(out, el, depth + 1);
        }
    }

    fn write_while(&mut self, out: &mut String, s: &WhileStmt, depth: usize) {
        self.indent(out, depth);
        write!(out, "while ").unwrap();
        self.write_expr(out, &s.condition);
        out.push_str(":\n");
        self.write_block(out, &s.body, depth + 1);
    }

    fn write_for(&mut self, out: &mut String, s: &ForStmt, depth: usize) {
        self.indent(out, depth);
        if let Some(ref idx_var) = s.index_variable {
            write!(out, "for {}, {} in ", idx_var, s.variable).unwrap();
        } else {
            write!(out, "for {} in ", s.variable).unwrap();
        }
        self.write_expr(out, &s.iterable);
        out.push_str(":\n");
        self.write_block(out, &s.body, depth + 1);
    }

    fn write_pattern(&mut self, out: &mut String, pattern: &Pattern) {
        match pattern {
            Pattern::Identifier { name, .. } => out.push_str(name),
            Pattern::Literal { value, .. } => self.write_literal(out, value),
            Pattern::Wildcard { .. } => out.push('_'),
            Pattern::Tuple { elements, .. } => {
                out.push('(');
                for (i, e) in elements.iter().enumerate() {
                    if i > 0 { out.push_str(", "); }
                    self.write_pattern(out, e);
                }
                out.push(')');
            }
            Pattern::EnumVariant { enum_name, variant, args, .. } => {
                write!(out, "{}.{}", enum_name, variant).unwrap();
                if !args.is_empty() {
                    out.push('(');
                    for (i, a) in args.iter().enumerate() {
                        if i > 0 { out.push_str(", "); }
                        self.write_pattern(out, a);
                    }
                    out.push(')');
                }
            }
            Pattern::IsType { type_, .. } => {
                write!(out, "is ").unwrap();
                self.write_type(out, type_);
            }
            Pattern::Range { start, end, inclusive, .. } => {
                self.write_literal(out, start);
                if *inclusive { out.push_str("..="); } else { out.push_str("..<"); }
                self.write_literal(out, end);
            }
            Pattern::Or { patterns, .. } => {
                for (i, p) in patterns.iter().enumerate() {
                    if i > 0 { out.push_str(" | "); }
                    self.write_pattern(out, p);
                }
            }
        }
    }

    fn write_match(&mut self, out: &mut String, s: &MatchStmt, depth: usize) {
        self.indent(out, depth);
        write!(out, "match ").unwrap();
        self.write_expr(out, &s.expression);
        out.push_str(":\n");
        for arm in &s.arms {
            self.indent(out, depth + 1);
            self.write_pattern(out, &arm.pattern);
            if let Some(guard) = &arm.guard {
                write!(out, " if ").unwrap();
                self.write_expr(out, guard);
            }
            out.push_str(":\n");
            for stmt in &arm.body.statements {
                self.write_stmt(out, stmt, depth + 2);
            }
        }
    }

    fn write_expr(&mut self, out: &mut String, expr: &Expr) {
        match expr {
            Expr::Literal { value, .. } => self.write_literal(out, value),
            Expr::Identifier { name, .. } => out.push_str(name),
            Expr::Binary { left, operator, right, .. } => {
                self.write_expr(out, left);
                write!(out, " {} ", self.binop_str(operator)).unwrap();
                self.write_expr(out, right);
            }
            Expr::Unary { operator, operand, .. } => {
                match operator {
                    UnaryOp::Neg => out.push('-'),
                    UnaryOp::Not => out.push_str("not "),
                    UnaryOp::BitNot => out.push('~'),
                }
                self.write_expr(out, operand);
            }
            Expr::FunctionCall { target, arguments, .. } => {
                self.write_expr(out, target);
                out.push('(');
                for (i, arg) in arguments.iter().enumerate() {
                    if i > 0 { out.push_str(", "); }
                    self.write_expr(out, arg);
                }
                out.push(')');
            }
            Expr::Assignment { target, value, .. } => {
                self.write_expr(out, target);
                write!(out, " = ").unwrap();
                self.write_expr(out, value);
            }
            Expr::PropertyAccess { object, property, .. } => {
                self.write_expr(out, object);
                write!(out, ".{}", property).unwrap();
            }
            Expr::OptionalChain { target, property, .. } => {
                self.write_expr(out, target);
                write!(out, "?.{}", property).unwrap();
            }
            Expr::ErrorProp { expression, .. } => {
                self.write_expr(out, expression);
                out.push('?');
            }
            Expr::StringInterp { parts, .. } => {
                out.push('"');
                for part in parts {
                    if let Expr::Literal { value: Literal::String(s), .. } = part {
                        out.push_str(s);
                    } else {
                        out.push('{');
                        self.write_expr(out, part);
                        out.push('}');
                    }
                }
                out.push('"');
            }
            Expr::IfExpr { .. } => unreachable!("desugared in HIR"),
            Expr::Ternary { cond, then_expr, else_expr, .. } => {
                self.write_expr(out, cond);
                out.push_str(" ? ");
                self.write_expr(out, then_expr);
                out.push_str(" : ");
                self.write_expr(out, else_expr);
            }
            Expr::MatchExpr { expression, arms, .. } => {
                out.push_str("match ");
                self.write_expr(out, expression);
                out.push_str(":\n");
                for arm in arms {
                    self.indent(out, 1);
                    self.write_pattern(out, &arm.pattern);
                    if let Some(g) = &arm.guard {
                        write!(out, " if ").unwrap();
                        self.write_expr(out, g);
                    }
                    out.push_str(":\n");
                    for stmt in &arm.body.statements {
                        self.write_stmt(out, stmt, 2);
                    }
                }
            }
            Expr::Array { elements, .. } => {
                out.push('[');
                for (i, e) in elements.iter().enumerate() {
                    if i > 0 { out.push_str(", "); }
                    self.write_expr(out, e);
                }
                out.push(']');
            }
            Expr::ArrayRepeat { value, count, .. } => {
                out.push('[');
                self.write_expr(out, value);
                out.push_str("; ");
                self.write_expr(out, count);
                out.push(']');
            }
            Expr::SetLiteral { collection_name, elements, .. } => {
                out.push_str(&format!("{}{{", collection_name));
                for (i, e) in elements.iter().enumerate() {
                    if i > 0 { out.push_str(", "); }
                    self.write_expr(out, e);
                }
                out.push('}');
            }
            Expr::List { elements, .. } => {
                out.push('[');
                for (i, e) in elements.iter().enumerate() {
                    if i > 0 { out.push_str(", "); }
                    self.write_expr(out, e);
                }
                out.push(']');
            }
            Expr::Dictionary { entries, .. } => {
                out.push('{');
                for (i, (k, v)) in entries.iter().enumerate() {
                    if i > 0 { out.push_str(", "); }
                    out.push_str(k);
                    out.push_str(": ");
                    self.write_expr(out, v);
                }
                out.push('}');
            }
            Expr::StructLiteral { struct_name, type_args: _, fields, .. } => {
                out.push_str(struct_name);
                out.push(' ');
                out.push('{');
                for (i, (k, v)) in fields.iter().enumerate() {
                    if i > 0 { out.push_str(", "); }
                    out.push_str(k);
                    out.push_str(": ");
                    self.write_expr(out, v);
                }
                out.push('}');
            }
            Expr::Tuple { elements, .. } => {
                out.push('(');
                for (i, e) in elements.iter().enumerate() {
                    if i > 0 { out.push_str(", "); }
                    self.write_expr(out, e);
                }
                out.push(')');
            }
            Expr::Closure { params, body, .. } => {
                out.push('(');
                for (i, (pname, ptype)) in params.iter().enumerate() {
                    if i > 0 { out.push_str(", "); }
                    out.push_str(pname);
                    if let Some(t) = ptype {
                        write!(out, ": {}", t).unwrap();
                    }
                }
                out.push_str(") => ");
                self.write_expr(out, body);
            }
            Expr::Await { expression, .. } => {
                write!(out, "await ").unwrap();
                self.write_expr(out, expression);
            }
            Expr::Async { expression, .. } => {
                write!(out, "async ").unwrap();
                self.write_expr(out, expression);
            }
            Expr::AsyncBlock { body, .. } => {
                write!(out, "async:\n").unwrap();
                self.write_block(out, body, 0);
            }
            Expr::Spread { expression, .. } => {
                write!(out, "..").unwrap();
                self.write_expr(out, expression);
            }
            Expr::Index { target, index, .. } => {
                self.write_expr(out, target);
                out.push('[');
                self.write_expr(out, index);
                out.push(']');
            }
            Expr::RangeSlice { target, start, end, .. } => {
                self.write_expr(out, target);
                out.push('[');
                if let Some(s) = start {
                    self.write_expr(out, s);
                }
                out.push_str("..");
                if let Some(e) = end {
                    self.write_expr(out, e);
                }
                out.push(']');
            }
            Expr::Loop { body, .. } => {
                out.push_str("loop:\n");
                self.write_block(out, body, 1);
            }
            Expr::BorrowRef { expression, .. } => self.write_expr(out, expression),
            Expr::NullCoalesce { left, right, .. } => self.write_expr(out, left),
        }
    }

    fn write_literal(&mut self, out: &mut String, lit: &Literal) {
        match lit {
            Literal::Integer(n) => write!(out, "{}", n).unwrap(),
            Literal::Float(n) => write!(out, "{}", n).unwrap(),
            Literal::String(s) => {
                out.push('"');
                out.push_str(&s.replace('\\', "\\\\").replace('"', "\\\""));
                out.push('"');
            }
            Literal::Boolean(b) => out.push_str(if *b { "true" } else { "false" }),
            Literal::Char(c) => out.push_str(&format!("'{}'", char::from_u32(*c as u32).unwrap_or('\0'))),
            Literal::None => out.push_str("None"),
            Literal::Null => out.push_str("null"),
        }
    }

    fn binop_str(&self, op: &BinaryOp) -> &'static str {
        match op {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::Rem => "%",
            BinaryOp::Pow => "**",
            BinaryOp::AddPercent => "+%",
            BinaryOp::SubPercent => "-%",
            BinaryOp::MulPercent => "*%",
            BinaryOp::Eq => "==",
            BinaryOp::Neq => "!=",
            BinaryOp::Lt => "<",
            BinaryOp::Gt => ">",
            BinaryOp::Le => "<=",
            BinaryOp::Ge => ">=",
            BinaryOp::And => "&&",
            BinaryOp::Or => "||",
            BinaryOp::BitAnd => "&",
            BinaryOp::BitOr => "|",
            BinaryOp::BitXor => "^",
            BinaryOp::Shl => "<<",
            BinaryOp::Shr => ">>",
            BinaryOp::Is => "is",
            BinaryOp::As => "as",
            BinaryOp::Assign => "=",
            BinaryOp::AddAssign => "+=",
            BinaryOp::SubAssign => "-=",
            BinaryOp::MulAssign => "*=",
            BinaryOp::DivAssign => "/=",
            BinaryOp::RemAssign => "%=",
            BinaryOp::BitAndAssign => "&=",
            BinaryOp::BitOrAssign => "|=",
            BinaryOp::BitXorAssign => "^=",
            BinaryOp::ShlAssign => "<<=",
            BinaryOp::ShrAssign => ">>=",
            BinaryOp::Range => "..",
            BinaryOp::RangeInclusive => "..=",
            BinaryOp::RangeExclusive => "..<",
        }
    }
}

/// Sort key: `(is_relative, module_name)`. Relative imports (with `~`)
/// sort before absolute imports; within each group, alphabetical by module name.
fn import_sort_key(decl: &Decl) -> (u8, &str) {
    match decl {
        Decl::Use(u) => (if u.relative { 0 } else { 1 }, u.path.last().map_or("", |s| s.as_str())),
        Decl::Import(i) => (if i.relative { 0 } else { 1 }, i.module_name.as_str()),
        Decl::FromImport(fi) => (if fi.relative { 0 } else { 1 }, fi.module_name.as_str()),
        _ => (2, ""),
    }
}

fn decl_span(decl: &Decl) -> kyc_core::span::Span {
    match decl {
        Decl::Use(u) => u.span,
        Decl::Import(i) => i.span,
        Decl::FromImport(fi) => fi.span,
        Decl::Variable(v) => v.span,
        Decl::Constant(c) => c.span,
        Decl::Function(f) => f.span,
        Decl::Class(c) => c.span,
        Decl::AbstractClass(c) => c.span,
        Decl::Struct(s) => s.span,
        Decl::Enum(e) => e.span,
        Decl::Contract(c) => c.span,
        Decl::TypeAlias(t) => t.span,
        Decl::Link(_, _) => kyc_core::span::Span::dummy(),
        Decl::Expression(_) => kyc_core::span::Span::dummy(),
    }
}

fn stmt_span(stmt: &Stmt) -> kyc_core::span::Span {
    match stmt {
        Stmt::Expression(e) => expr_span(e),
        Stmt::Variable(v) => v.span,
        Stmt::TypedVariable(v) => v.span,
        Stmt::Return(_) => kyc_core::span::Span::dummy(),
        Stmt::If(s) => s.span,
        Stmt::While(s) => s.span,
        Stmt::For(s) => s.span,
        Stmt::Match(s) => s.span,
        Stmt::Guard(s) => s.span,
        Stmt::Unsafe(s) => s.span,
        Stmt::Defer(s) => s.span,
        Stmt::BindingIf(s) => s.span,
        Stmt::Break(_, _) => kyc_core::span::Span::dummy(),
        Stmt::Continue(_) => kyc_core::span::Span::dummy(),
        Stmt::WhileBind(s) => s.span,
        Stmt::Constant(c) => c.span,
    }
}

fn expr_span(expr: &Expr) -> kyc_core::span::Span {
    match expr {
        Expr::Literal { span, .. } => *span,
        Expr::Identifier { span, .. } => *span,
        Expr::Binary { span, .. } => *span,
        Expr::Unary { span, .. } => *span,
        Expr::FunctionCall { span, .. } => *span,
        Expr::Assignment { span, .. } => *span,
        Expr::PropertyAccess { span, .. } => *span,
        Expr::OptionalChain { span, .. } => *span,
        Expr::ErrorProp { span, .. } => *span,
        Expr::Index { span, .. } => *span,
        Expr::List { span, .. } => *span,
        Expr::Array { span, .. } => *span,
        Expr::ArrayRepeat { span, .. } => *span,
        Expr::Dictionary { span, .. } => *span,
        Expr::SetLiteral { span, .. } => *span,
        Expr::List { span, .. } => *span,
        Expr::Array { span, .. } => *span,
        Expr::Tuple { span, .. } => *span,
        Expr::Closure { span, .. } => *span,
        Expr::Await { span, .. } => *span,
        Expr::Async { span, .. } => *span,
        Expr::AsyncBlock { span, .. } => *span,
        Expr::Spread { span, .. } => *span,
        Expr::RangeSlice { span, .. } => *span,
        Expr::Loop { span, .. } => *span,
        Expr::StructLiteral { span, .. } => *span,
        Expr::StringInterp { span, .. } => *span,
        Expr::IfExpr { .. } => unreachable!("desugared in HIR"),
            Expr::Ternary { span, .. } => *span,
        Expr::MatchExpr { span, .. } => *span,
        Expr::BorrowRef { span, .. } => *span,
        Expr::NullCoalesce { span, .. } => *span,
    }
}
