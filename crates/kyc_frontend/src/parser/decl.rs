use kyc_core::ast::*;
use kyc_core::span::Span;
use crate::token::{Token, TokenKind};

use super::Parser;

impl Parser {
    pub(crate) fn parse_decl(&mut self) -> Result<Decl, String> {
        if self.at(TokenKind::Use) {
            return self.parse_use();
        }
        if self.at(TokenKind::From) {
            return self.parse_from_import();
        }
        if self.at(TokenKind::Import) {
            return self.parse_import();
        }
        if self.at(TokenKind::Type) {
            return self.parse_type_alias();
        }
        // `#[test]` attribute before function
        if self.at(TokenKind::Hash) {
            return self.parse_attr_function();
        }
        // extern fn — declare external C function (no body)
        if self.at(TokenKind::Extern) {
            let start = self.pos;
            self.advance();
            if !self.at(TokenKind::Fn) {
                return Err("expected 'fn' after 'extern'".to_string());
            }
            self.advance();
            let raw_name = self.eat_identifier();
            let (name, _visibility) = if raw_name.starts_with("__") {
                (raw_name.trim_start_matches('_').to_string(), Visibility::Private)
            } else if raw_name.starts_with('_') {
                (raw_name.trim_start_matches('_').to_string(), Visibility::Protected)
            } else {
                (raw_name.clone(), Visibility::Public)
            };
            self.expect(TokenKind::LParen)?;
            let params = self.parse_params()?;
            self.expect(TokenKind::RParen)?;
            let return_type = if self.at(TokenKind::Colon) || self.at(TokenKind::Newline)
                || self.at(TokenKind::Dedent) || self.at(TokenKind::Eof)
            {
                None
            } else {
                Some(self.parse_type()?)
            };
            return Ok(Decl::Function(FunctionDecl {
                name, params, return_type,
                type_params: vec![],
                is_async: false, is_const: false, is_abstract: false,
                is_static: false, is_test: false, is_bench: false, is_extern: true,
                visibility: Visibility::Public,
                body: None,
                span: self.span_from(start),
            }));
        }
        // @link "libname" — link against a native library
        if self.at(TokenKind::At) {
            let start = self.pos;
            self.advance();
            if self.at_identifier() && self.eat_identifier() == "link" {
                if let TokenKind::String(s) = &self.current()?.kind {
                    let name = s.clone();
                    self.advance();
                    self.links.push(name.clone());
                    return Ok(Decl::Link(name, self.span_from(start)));
                }
                return Err("expected string literal after '@link'".to_string());
            }
            self.pos = start; // backtrack — not @link
        }
        if self.at(TokenKind::Fn)
            || self.at(TokenKind::Async)
            || self.at(TokenKind::Const)
        {
            return self.parse_function(false).map(Decl::Function);
        }
        // Class: `final class X:`, `class X:`, `abstract class X:`
        if self.at(TokenKind::Final) {
            self.advance();
            if self.at(TokenKind::Class) {
                return self.parse_class(false); // final class = normal class
            }
            return Err("expected 'class' after 'final'".to_string());
        }
        if self.at(TokenKind::Abstract) {
            self.advance();
            if self.at(TokenKind::Class) {
                return self.parse_class(true);
            }
            return Err("expected 'class' after 'abstract'".to_string());
        }
        if self.at(TokenKind::Class) {
            return self.parse_class(false);
        }
        if self.at(TokenKind::Struct) {
            return self.parse_struct().map(Decl::Struct);
        }
        if self.at(TokenKind::Enum) {
            return self.parse_enum().map(Decl::Enum);
        }
        if self.at(TokenKind::Contract) {
            return self.parse_contract().map(Decl::Contract);
        }
        // Variable/constant declaration: `name = expr`, `name: &type = expr`, or `NAME := expr`
        if self.at_identifier() {
            let start = self.pos;
            let name = self.eat_identifier();
            // Check for typed declaration: `ident : type = expr`
            if self.at(TokenKind::Colon) {
                self.advance();
                let type_ = self.parse_type()?;
                let is_mutable = matches!(type_, AstType::Mutable { .. });
                if self.at(TokenKind::Equals) {
                    self.advance();
                    let value = self.parse_expr()?;
                    return Ok(Decl::Variable(VariableDecl {
                        name, type_: Some(type_), value: Box::new(value), is_mutable, span: self.span_from(start),
                    }));
                }
                return Err("expected '=' after typed declaration".to_string());
            }
            if self.at(TokenKind::Walrus) {
                // `NAME := expr` → constant
                self.advance();
                let value = self.parse_expr()?;
                return Ok(Decl::Constant(ConstantDecl {
                    name, value: Box::new(value), span: self.span_from(start),
                }));
            } else if self.at(TokenKind::Equals) {
                self.advance();
                let value = self.parse_expr()?;
                return Ok(Decl::Variable(VariableDecl {
                    name, type_: None, value: Box::new(value), is_mutable: false, span: self.span_from(start),
                }));
            }
            // Not a variable/constant declaration — try as module-level expression
            // e.g. `app.get("/users", handler)` at top level
            self.pos = start;
            if let Ok(expr) = self.parse_binary(0) {
                return Ok(Decl::Expression(expr));
            }
            return Err(format!("expected '=', ':=', or ': type =' after identifier '{}'", name));
        }
        let found = self.current().map(|t| format!("{:?}", t.kind)).unwrap_or_else(|_| "EOF".into());
        Err(format!("unexpected token at declaration start: {}", found))
    }

    /// Read a dotted name: ident . ident . ident ...
    pub(crate) fn read_dotted_name(&mut self) -> Result<String, String> {
        let mut name = self.eat_identifier();
        while self.at(TokenKind::Dot) {
            self.advance(); // consume .
            name.push('.');
            name.push_str(&self.eat_identifier());
        }
        Ok(name)
    }

    pub(crate) fn parse_use(&mut self) -> Result<Decl, String> {
        let start = self.pos;
        self.advance(); // use
        let mut relative = false;
        // Check for leading dot for relative import: use .helper or use ..parent
        if self.at(TokenKind::Dot) {
            relative = true;
        } else if self.at(TokenKind::Tilde) {
            self.advance();
            relative = true;
        }
        // Read dotted path, stopping at { or * or newline
        let mut path = Vec::new();
        if !relative {
            path.push(self.eat_identifier());
        }
        while self.at(TokenKind::Dot) {
            self.advance(); // .
            // Wildcard: use my.*
            if self.at(TokenKind::Star) {
                self.advance();
                return Ok(Decl::Use(UseDecl { path, names: Some(vec!["*".to_string()]), alias: None, relative, span: self.span_from(start) }));
            }
            // Peek at next without consuming: if it's an identifier or keyword-accepted-as-ident, read it
            if let Ok(tok) = self.current() {
                let is_ident = matches!(&tok.kind,
                    TokenKind::Identifier(_) | TokenKind::Get | TokenKind::Set
                    | TokenKind::None | TokenKind::True | TokenKind::False
                    | TokenKind::Type | TokenKind::For | TokenKind::In | TokenKind::Is
                    | TokenKind::As
                );
                if is_ident {
                    path.push(self.eat_identifier());
                    continue;
                }
            }
            // For .helper pattern: if relative and path is empty, read the identifier
            if relative && path.is_empty() {
                path.push(self.eat_identifier());
                continue;
            }
            // Not an identifier — could be { for selective import, or end of line.
            break;
        }
        // Check for selective import: use X.Y.{name1, name2}
        if self.at(TokenKind::LBrace) {
            self.expect(TokenKind::LBrace)?;
            let mut names = Vec::new();
            names.push(self.eat_identifier());
            while self.at(TokenKind::Comma) {
                self.advance();
                names.push(self.eat_identifier());
            }
            self.expect(TokenKind::RBrace)?;
            return Ok(Decl::Use(UseDecl { path, names: Some(names), alias: None, relative, span: self.span_from(start) }));
        }
        let alias = if self.at(TokenKind::As) {
            self.advance();
            Some(self.eat_identifier())
        } else {
            None
        };
        Ok(Decl::Use(UseDecl { path, names: None, alias, relative, span: self.span_from(start) }))
    }

    pub(crate) fn parse_import(&mut self) -> Result<Decl, String> {
        let start = self.pos;
        self.advance(); // import
        let relative = if self.at(TokenKind::Tilde) {
            self.advance();
            true
        } else {
            false
        };
        let module_name = self.read_dotted_name()?;
        let alias = if self.at(TokenKind::As) {
            self.advance();
            Some(self.eat_identifier())
        } else {
            None
        };
        Ok(Decl::Import(Import { module_name, alias, relative, span: self.span_from(start) }))
    }

    pub(crate) fn parse_from_import(&mut self) -> Result<Decl, String> {
        let start = self.pos;
        self.advance(); // from
        let relative = if self.at(TokenKind::Tilde) {
            self.advance();
            true
        } else {
            false
        };
        let module_name = self.read_dotted_name()?;
        self.expect_keyword("import")?;
        let mut imported_names = Vec::new();
        imported_names.push(self.eat_identifier());
        while self.at(TokenKind::Comma) {
            self.advance(); // ,
            imported_names.push(self.eat_identifier());
        }
        let alias = if imported_names.len() == 1 && self.at(TokenKind::As) {
            self.advance();
            Some(self.eat_identifier())
        } else {
            None
        };
        Ok(Decl::FromImport(FromImport { module_name, imported_names, alias, relative, span: self.span_from(start) }))
    }

    pub(crate) fn parse_type_alias(&mut self) -> Result<Decl, String> {
        let start = self.pos;
        self.advance(); // type
        let name = self.eat_identifier();
        let type_params = if self.at(TokenKind::Less) {
            self.parse_type_params()?
        } else {
            vec![]
        };
        if !self.at(TokenKind::Equals) {
            return Err("expected '=' in type alias".to_string());
        }
        self.advance();
        let type_ = self.parse_type()?;
        Ok(Decl::TypeAlias(TypeAlias { name, type_params, type_, span: self.span_from(start) }))
    }

    /// Parse `#[attr]` before a function declaration.
    /// Recognizes `#[test]` and `#[bench]`.
    pub(crate) fn parse_attr_function(&mut self) -> Result<Decl, String> {
        let start = self.pos;
        self.advance(); // consume #
        if !self.at(TokenKind::LBracket) {
            return Err("expected '[' after '#' for attribute".to_string());
        }
        self.advance(); // consume [
        let attr_name = self.eat_identifier();
        if !self.at(TokenKind::RBracket) {
            return Err("expected ']' after attribute name".to_string());
        }
        self.advance(); // consume ]
        let is_test = attr_name == "test";
        let is_bench = attr_name == "bench";
        if !is_test && !is_bench {
            return Err(format!("unknown attribute: #[{}]", attr_name));
        }
        // Skip newlines/indentation between attribute and function
        while self.at(TokenKind::Newline) || self.at(TokenKind::Indent) || self.at(TokenKind::Dedent) {
            self.advance();
        }
        let mut func = self.parse_function(is_test)?;
        func.is_test = is_test;
        func.is_bench = is_bench;
        func.span = self.span_from(start);
        Ok(Decl::Function(func))
    }

    pub(crate) fn parse_function(&mut self, is_test: bool) -> Result<FunctionDecl, String> {
        let start = self.pos;
        let is_const = if self.at(TokenKind::Const) { self.advance(); true } else { false };
        let is_async = if self.at(TokenKind::Async) { self.advance(); true } else { false };
        let is_static = if self.at(TokenKind::Static) { self.advance(); true } else { false };
        if !self.at(TokenKind::Fn) {
            return Err("expected 'fn'".to_string());
        }
        self.advance();
        let mut raw_name = self.eat_identifier();
        // Operator overloading: "op_" followed by operator token → "op_+", "op_-", etc.
        if raw_name == "op" || raw_name == "op_" {
            let op_sym = match self.current()?.kind {
                TokenKind::Plus => { self.advance(); "+" }
                TokenKind::Minus => { self.advance(); "-" }
                TokenKind::Star => { self.advance(); "*" }
                TokenKind::Slash => { self.advance(); "/" }
                TokenKind::Percent => { self.advance(); "%" }
                TokenKind::Equals => { self.advance(); "==" }
                TokenKind::BangEquals => { self.advance(); "!=" }
                TokenKind::Less => { self.advance(); "<" }
                TokenKind::Greater => { self.advance(); ">" }
                TokenKind::LessEquals => { self.advance(); "<=" }
                TokenKind::GreaterEquals => { self.advance(); ">=" }
                _ => "",
            };
            if !op_sym.is_empty() {
                raw_name = format!("op_{}", op_sym);
            }
        }
        // Visibility convention: `__` prefix → private, `_` prefix → protected, none → public.
        // The stored name is always stripped of the convention prefixes.
        let (name, visibility) = if raw_name.starts_with("__") {
            (raw_name.trim_start_matches('_').to_string(), Visibility::Private)
        } else if raw_name.starts_with('_') {
            (raw_name.trim_start_matches('_').to_string(), Visibility::Protected)
        } else {
            (raw_name.clone(), Visibility::Public)
        };
        // Parse optional type params: `<T, U>`
        let type_params = if self.at(TokenKind::Less) {
            self.parse_type_params()?
        } else {
            vec![]
        };
        self.expect(TokenKind::LParen)?;
        let params = self.parse_params()?;
        self.expect(TokenKind::RParen)?;
        // Support both `fn name() Type:` and `fn name() -> Type:` syntax
        if self.at(TokenKind::Arrow) {
            self.advance();
        }
        let return_type = if self.at(TokenKind::Colon)
            || self.at(TokenKind::Newline) || self.at(TokenKind::Dedent) || self.at(TokenKind::Eof)
        {
            None
        } else {
            let type_start = self.pos;
            let base = self.parse_type()?;
            if self.at(TokenKind::Bang) {
                self.advance();
                Some(AstType::Error { inner: Box::new(base), span: self.span_from(type_start) })
            } else {
                Some(base)
            }
        };
        let is_abstract = !self.at(TokenKind::Colon);
        let body = if is_abstract {
            None
        } else {
            self.advance(); // ':'
            Some(self.parse_block()?)
        };
        Ok(FunctionDecl {
            name, type_params, params, return_type, is_async, is_const, is_static, is_abstract, is_extern: false, is_test, is_bench: false, visibility, body,
            span: self.span_from(start),
        })
    }

    pub(crate) fn parse_class(&mut self, is_abstract: bool) -> Result<Decl, String> {
        let start = self.pos;
        self.advance(); // class
        let name = self.eat_identifier();
        // Parse optional type params: `<T, U>`
        let type_params = if self.at(TokenKind::Less) {
            self.parse_type_params()?
        } else {
            vec![]
        };

        // `class Name :: Parent, Contract1, Contract2:`
        // `::` is lexed as two consecutive Colons (not ConstDecl)
        let mut parent: Option<String> = None;
        let mut contracts: Vec<String> = Vec::new();

        if self.at(TokenKind::Colon) {
            let saved = self.pos;
            self.advance();
            if self.at(TokenKind::Colon) {  // second colon → :: syntax
                self.advance();
                // Parse comma-separated list of identifiers
                let mut items: Vec<String> = Vec::new();
                if self.at_identifier() {
                    items.push(self.eat_identifier());
                    while self.at(TokenKind::Comma) {
                        self.advance();
                        if self.at_identifier() {
                            items.push(self.eat_identifier());
                        } else { break; }
                    }
                }
                // First item = parent (if it exists), rest = contracts
                if !items.is_empty() {
                    parent = Some(items.remove(0));
                    contracts = items;
                }
            } else {
                // Single colon without :: — simple class, no parent/contracts
                self.pos = saved;
            }
        }

        // Expect body start
        self.expect(TokenKind::Colon)?;
        let members = self.parse_class_members()?;
        self.make_class_decl(start, name, type_params, parent, contracts, members, is_abstract)
    }

    pub(crate) fn make_class_decl(
        &self,
        start_pos: usize,
        name: String,
        type_params: Vec<TypeParam>,
        parent: Option<String>,
        contracts: Vec<String>,
        mut members: Vec<ClassMember>,
        is_abstract: bool,
    ) -> Result<Decl, String> {
        // If the class has no explicit constructor, synthesize a default empty
        // one so that `ClassName()` instantiations work without user-written
        // boilerplate.
        let has_ctor = members.iter().any(|m| matches!(m, ClassMember::Constructor(_)));
        if !has_ctor && !is_abstract {
            members.push(ClassMember::Constructor(Constructor {
                params: Vec::new(),
                body: Block { statements: Vec::new(), span: self.span_from(start_pos) },
                span: self.span_from(start_pos),
            }));
        }
        if is_abstract {
            Ok(Decl::AbstractClass(AbstractClassDecl {
                name, type_params, parent, contracts, members, span: self.span_from(start_pos),
            }))
        } else {
            Ok(Decl::Class(ClassDecl {
                name, type_params, parent, contracts, members, span: self.span_from(start_pos),
            }))
        }
    }

    pub(crate) fn parse_class_members(&mut self) -> Result<Vec<ClassMember>, String> {
        // Consume the Newline and Indent that precede the class body.
        if self.at(TokenKind::Newline) { self.advance(); }
        if self.at(TokenKind::Indent) { self.advance(); }
        let mut members = Vec::new();
        loop {
            if self.at(TokenKind::Dedent) { self.advance(); break; }
            if self.at(TokenKind::Eof) { break; }
            if self.at(TokenKind::Newline) { self.advance(); continue; }
            // Constructor: `Name(params):`
            if self.at_identifier() {
        let name = self.eat_identifier();
                // Check for constructor: identifier followed by '('
                if self.at(TokenKind::LParen) {
                    let constructor_start = self.pos;
                    self.advance();
                    let params = self.parse_params()?;
                    self.expect(TokenKind::RParen)?;
                    self.expect(TokenKind::Colon)?;
                    let body = self.parse_block()?;
                    members.push(ClassMember::Constructor(Constructor {
                        params, body, span: self.span_from(constructor_start),
                    }));
                    continue;
                }
                // Check for field or property: identifier followed by ':'
                if self.at(TokenKind::Colon) {
                    let member_start = self.pos;
                    self.advance();
                    let type_ = self.parse_type()?;
                    let is_mutable = matches!(type_, AstType::Mutable { .. });
                    let visibility = if name.starts_with("__") {
                        Visibility::Private
                    } else if name.starts_with('_') {
                        Visibility::Protected
                    } else {
                        Visibility::Public
                    };
                    // Field default: `name: type = expr`
                    let default = if self.at(TokenKind::Equals) {
                        self.advance();
                        Some(Box::new(self.parse_expr()?))
                    } else {
                        None
                    };
                    // Check for property getter/setter blocks (indented under the field)
                    let mut getter = None;
                    let mut setter = None;
                    if self.at(TokenKind::Newline) {
                        self.advance();
                        if self.at(TokenKind::Indent) {
                            self.advance(); // enter property block
                            loop {
                                if self.at(TokenKind::Newline) { self.advance(); continue; }
                                if self.at(TokenKind::Dedent) { self.advance(); break; }
                                if self.at(TokenKind::Get) {
                                    self.advance();
                                    self.expect(TokenKind::Colon)?;
                                    getter = Some(self.parse_block()?);
                                } else if self.at(TokenKind::Set) {
                                    self.advance();
                                    let set_param = if self.at(TokenKind::LParen) {
                                        self.advance();
                                        let p = self.eat_identifier();
                                        self.expect(TokenKind::RParen)?;
                                        p
                                    } else {
                                        "value".to_string()
                                    };
                                    self.expect(TokenKind::Colon)?;
                                    setter = Some((set_param, self.parse_block()?));
                                } else {
                                    break;
                                }
                            }
                        }
                    }
                    if getter.is_some() || setter.is_some() {
                        members.push(ClassMember::Property(Property {
                            name, type_, getter, setter, span: self.span_from(member_start),
                        }));
                    } else {
                        members.push(ClassMember::Field(Field {
                            name, type_, is_mutable, default, visibility, span: self.span_from(member_start),
                        }));
                    }
                    continue;
                }
                // Unknown — break and let the outer parser handle it
                break;
            }
            // Method: `static fn name(params):` or `fn name(params):`
            if self.at(TokenKind::Static) || self.at(TokenKind::Fn) || self.at(TokenKind::Async) || self.at(TokenKind::Const) || self.at(TokenKind::Abstract) {
                let is_static = self.at(TokenKind::Static);
                let mut method = self.parse_function(false)?;
                // Auto-prepend `this` parameter for instance methods (not static)
                // unless the method already has 'this' or 'self' as first param
                if !is_static {
                    let has_this = method.params.first().map_or(false, |p| p.name == "this" || p.name == "self");
                    if !has_this {
                        let this_type = AstType::Primitive { name: "i32".into(), span: method.span.clone() };
                        method.params.insert(0, Parameter {
                            name: "this".into(),
                            type_: this_type,
                            default: None, variadic: false,
                            mode: ParamMode::MutableBorrow, span: method.span,
                        });
                    }
                }
                members.push(ClassMember::Method(method));
                continue;
            }
            break;
        }
        Ok(members)
    }

    pub(crate) fn parse_struct(&mut self) -> Result<StructDecl, String> {
        let start = self.pos;
        self.advance(); // struct
        let name = self.eat_identifier();
        let type_params = if self.at(TokenKind::Less) {
            self.parse_type_params()?
        } else {
            vec![]
        };
        self.expect(TokenKind::Colon)?;
        if self.at(TokenKind::Newline) { self.advance(); }
        if self.at(TokenKind::Indent) { self.advance(); }
        let mut fields = Vec::new();
        loop {
            if self.at(TokenKind::Dedent) { self.advance(); break; }
            if self.at(TokenKind::Eof) { break; }
            if self.at(TokenKind::Newline) { self.advance(); continue; }
            let field_start = self.pos;
            let field_name = self.eat_identifier();
            self.expect(TokenKind::Colon)?;
            let type_ = self.parse_type()?;
            let is_mutable = matches!(type_, AstType::Mutable { .. });
            fields.push(Field { name: field_name, type_, is_mutable, default: None, visibility: Visibility::Public, span: self.span_from(field_start) });
        }
        Ok(StructDecl { name, type_params, fields, span: self.span_from(start) })
    }

    pub(crate) fn parse_enum(&mut self) -> Result<EnumDecl, String> {
        let start = self.pos;
        self.advance(); // enum
        let name = self.eat_identifier();
        let type_params = if self.at(TokenKind::Less) {
            self.parse_type_params()?
        } else {
            vec![]
        };
        self.expect(TokenKind::Colon)?;
        if self.at(TokenKind::Newline) { self.advance(); }
        if self.at(TokenKind::Indent) { self.advance(); }
        let mut variants = Vec::new();
        loop {
            if self.at(TokenKind::Dedent) { self.advance(); break; }
            if self.at(TokenKind::Eof) { break; }
            if self.at(TokenKind::Newline) { self.advance(); continue; }
            let variant_start = self.pos;
            let variant_name = self.eat_identifier();
            if variant_name.is_empty() {
                let found = format!("{:?}", self.current()?.kind);
                return Err(format!("expected enum variant name, found {}", found));
            }
            let payload = if self.at(TokenKind::LParen) {
                self.advance();
                let mut types = Vec::new();
                loop {
                    types.push(self.parse_type()?);
                    if self.at(TokenKind::Comma) { self.advance(); }
                    else { break; }
                }
                self.expect(TokenKind::RParen)?;
                types
            } else {
                vec![]
            };
            variants.push(EnumVariant { name: variant_name, payload, span: self.span_from(variant_start) });
        }
        Ok(EnumDecl { name, type_params, variants, span: self.span_from(start) })
    }

    pub(crate) fn parse_contract(&mut self) -> Result<ContractDecl, String> {
        let start = self.pos;
        self.advance(); // contract
        let name = self.eat_identifier();
        self.expect(TokenKind::Colon)?;
        if self.at(TokenKind::Newline) { self.advance(); }
        if self.at(TokenKind::Indent) { self.advance(); }
        let mut methods = Vec::new();
        loop {
            if self.at(TokenKind::Dedent) { self.advance(); break; }
            if self.at(TokenKind::Eof) { break; }
            if self.at(TokenKind::Newline) { self.advance(); continue; }
            if !self.at(TokenKind::Fn) { break; }
            let method_start = self.pos;
            self.advance();
            let method_name = self.eat_identifier();
            self.expect(TokenKind::LParen)?;
            let params = self.parse_params()?;
            self.expect(TokenKind::RParen)?;
            let return_type = if self.at(TokenKind::Colon)
                || self.at(TokenKind::Newline) || self.at(TokenKind::Dedent) || self.at(TokenKind::Eof)
            {
                None
            } else {
                Some(self.parse_type()?)
            };
            methods.push(ContractMethod { name: method_name, params, return_type, span: self.span_from(method_start) });
        }
        Ok(ContractDecl { name, methods, span: self.span_from(start) })
    }
}
