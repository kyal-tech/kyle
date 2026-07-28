use kyc_core::ast::*;
use kyc_core::span::Span;
use crate::token::{Token, TokenKind};

use super::Parser;

impl Parser {
    pub(crate) fn parse_params(&mut self) -> Result<Vec<Parameter>, String> {
        let mut params = Vec::new();
        loop {
            if self.at(TokenKind::RParen) { break; }
            let param_start = self.pos;
            // `^name: type` — move/ownership transfer parameter
            let is_move = self.at(TokenKind::Caret);
            if is_move {
                self.advance();
            }
            let variadic = if self.at(TokenKind::DotDotDot) {
                self.advance();
                true
            } else {
                false
            };
            let name = if variadic {
                if self.at_identifier() {
                    self.eat_identifier()
                } else {
                    "_args".to_string()
                }
            } else {
                self.eat_identifier()
            };
            let type_ = if self.at(TokenKind::Colon) {
                self.advance();
                self.parse_type()?
            } else {
                AstType::Primitive { name: "i64".into(), span: self.span_from(param_start) }
            };
            // Determine param mode from type prefix
            // Kyle: parameters are BORROWED by default (different from Rust's move-by-default)
            let mode = if is_move {
                ParamMode::Move
            } else if matches!(type_, AstType::Borrow { .. }) {
                ParamMode::Borrow
            } else {
                let is_mut_borrow = match &type_ {
                    AstType::Mutable { inner, .. } => matches!(inner.as_ref(), AstType::Borrow { .. }),
                    _ => false,
                };
                if is_mut_borrow { ParamMode::MutableBorrow }
                else { ParamMode::Borrow }
            };
            let default = if self.at(TokenKind::Equals) {
                self.advance();
                Some(Box::new(self.parse_expr()?))
            } else { None };
            params.push(Parameter { name, type_, default, variadic, mode, span: self.span_from(param_start) });
            if self.at(TokenKind::Comma) { self.advance(); } else { break; }
        }
        Ok(params)
    }

    pub(crate) fn parse_type_params(&mut self) -> Result<Vec<TypeParam>, String> {
        self.advance(); // '<'
        let mut params = Vec::new();
        loop {
            if self.at(TokenKind::Greater) { break; }
            let param_start = self.pos;
            let name = self.eat_identifier();
            let constraint = if self.at(TokenKind::Colon) {
                self.advance();
                Some(self.parse_type()?)
            } else {
                None
            };
            params.push(TypeParam { name, constraint, span: self.span_from(param_start) });
            if self.at(TokenKind::Comma) { self.advance(); } else { break; }
        }
        self.expect(TokenKind::Greater)?;
        Ok(params)
    }

    // -----------------------------------------------------------------------
    // Type parsing
    // -----------------------------------------------------------------------

    pub(crate) fn parse_type(&mut self) -> Result<AstType, String> {
        let start = self.pos;
        // Handle `&T` — borrow type, or `&[T]` — slice type
        if self.at(TokenKind::Ampersand) {
            self.advance();
            if self.at(TokenKind::LBracket) {
                self.advance();
                let inner = self.parse_type()?;
                self.expect(TokenKind::RBracket)?;
                return Ok(AstType::Slice { inner: Box::new(inner), span: self.span_from(start) });
            }
            let inner = self.parse_type()?;
            return Ok(AstType::Borrow { inner: Box::new(inner), span: self.span_from(start) });
        }
        // Handle `^T` — mutable type, and `^&T` — mutable borrow
        if self.at(TokenKind::Caret) {
            self.advance();
            if self.at(TokenKind::Ampersand) {
                // `^&T` — mutable borrow (NOT slice, that requires just `&[T]` without `^`)
                self.advance();
                let inner = self.parse_type()?;
                let borrow = AstType::Borrow { inner: Box::new(inner), span: self.span_from(start) };
                return Ok(AstType::Mutable { inner: Box::new(borrow), span: self.span_from(start) });
            }
            let inner = self.parse_type()?;
            return Ok(AstType::Mutable { inner: Box::new(inner), span: self.span_from(start) });
        }
        // Handle list shorthand: [T] → List<T> with optional ? and ! postfix
        if self.at(TokenKind::LBracket) {
            self.advance();
            let inner = self.parse_type()?;
            if self.at(TokenKind::Comma) {
                self.advance();
                let size = self.parse_expr()?;  // parse integer expression for size
                let size_val = if let Expr::Literal { value: Literal::Integer(n), .. } = &size {
                    *n as usize
                } else { 0 };
                self.expect(TokenKind::RBracket)?;
                let mut base = AstType::Array { inner: Box::new(inner), size: size_val, span: self.span_from(start) };
                // Postfix ? and ! for [T, N]? and [T, N]!
                if self.at(TokenKind::Question) { self.advance(); base = AstType::Optional { inner: Box::new(base), span: self.span_from(start) }; }
                if self.at(TokenKind::Bang) { self.advance(); base = AstType::Error { inner: Box::new(base), span: self.span_from(start) }; }
                return Ok(base);
            }
            self.expect(TokenKind::RBracket)?;
            let mut base = AstType::List { inner: Box::new(inner), span: self.span_from(start) };
            // Postfix ? and ! for [T]? and [T]!
            if self.at(TokenKind::Question) { self.advance(); base = AstType::Optional { inner: Box::new(base), span: self.span_from(start) }; }
            if self.at(TokenKind::Bang) { self.advance(); base = AstType::Error { inner: Box::new(base), span: self.span_from(start) }; }
            return Ok(base);
        }
        // Handle function pointer type: fn(T, U) V
        if self.at(TokenKind::Fn) {
            self.advance(); // 'fn'
            self.expect(TokenKind::LParen)?;
            let mut elems = Vec::new();
            if !self.at(TokenKind::RParen) {
                elems.push(self.parse_type()?);
                while self.at(TokenKind::Comma) {
                    self.advance();
                    if self.at(TokenKind::RParen) { break; }
                    elems.push(self.parse_type()?);
                }
            }
            self.expect(TokenKind::RParen)?;
            // Optional return type (void if absent)
            let return_ = if self.at(TokenKind::Colon) || self.at(TokenKind::Newline)
                || self.at(TokenKind::Dedent) || self.at(TokenKind::Eof)
                || self.at(TokenKind::Comma) || self.at(TokenKind::RParen)
                || self.at(TokenKind::Greater) || self.at(TokenKind::RBracket)
            {
                None
            } else {
                Some(self.parse_type()?)
            };
            let return_type = return_.unwrap_or(
                AstType::Primitive { name: "void".to_string(), span: self.span_from(start) }
            );
            return Ok(AstType::FnPtr {
                params: elems,
                return_: Box::new(return_type),
                span: self.span_from(start),
            });
        }
        // Tuple type: (T, U) — only if not followed by more type syntax
        if self.at(TokenKind::LParen) {
            self.advance();
            let mut elems = Vec::new();
            if !self.at(TokenKind::RParen) {
                elems.push(self.parse_type()?);
                while self.at(TokenKind::Comma) {
                    self.advance();
                    if self.at(TokenKind::RParen) { break; }
                    elems.push(self.parse_type()?);
                }
            }
            self.expect(TokenKind::RParen)?;
            // Tuples are represented as a generic with name "tuple" for now
            return Ok(AstType::Generic {
                name: "tuple".to_string(),
                args: elems,
                span: self.span_from(start),
            });
        }
        // Handle {T} (list type) or {K: V} (dict type)
        if self.at(TokenKind::LBrace) {
            self.advance();
            let inner = self.parse_type()?;
            if self.at(TokenKind::Colon) {
                self.advance();
                let val_type = self.parse_type()?;
                self.expect(TokenKind::RBrace)?;
                return Ok(AstType::Dict { key: Box::new(inner), value: Box::new(val_type), span: self.span_from(start) });
            }
            self.expect(TokenKind::RBrace)?;
            return Ok(AstType::Generic { name: "list".to_string(), args: vec![inner], span: self.span_from(start) });
        }
        let name = self.eat_identifier();
        if name.is_empty() {
            let found = self.current().map(|t| format!("{:?}", t.kind)).unwrap_or_else(|_| "EOF".into());
            return Err(format!("expected type name, found {}", found));
        }
        // Handle ptr as a built-in type
        if name == "ptr" {
            return Ok(AstType::Ptr { span: self.span_from(start) });
        }
        // Handle bytes as a built-in type
        if name == "bytes" {
            return Ok(AstType::Bytes { span: self.span_from(start) });
        }
        let mut base = if self.at(TokenKind::Less) {
            self.advance();
            let mut args = Vec::new();
            args.push(self.parse_type()?);
            while self.at(TokenKind::Comma) {
                self.advance();
                args.push(self.parse_type()?);
            }
            // Accept either > or >> (>> is split: first > closes this generic,
            // the second remains in the token stream for the outer level)
            if self.at(TokenKind::Greater) {
                self.advance();
            } else if self.at(TokenKind::GreaterGreater) {
                self.advance(); // Consume >>
                // The remaining > from >> is lost — user must use space: list<i32> >
            } else {
                return Err("expected '>' after generic type arguments".to_string());
            }
            // If we consumed a >> (GreaterGreater) inside generic type args,
            // the next outer level check for > will see whatever follows.
            // For nested generics like list<i32>>, the >> is split as: first > closes list,
            // the second > remains for the outer level. But since >> is a single token,
            // we consumed both. To handle this properly, we'd need to push back a > token.
            // For now, use spaces: list<i32> > works, list<i32>> doesn't.
            if name == "set" && args.len() == 1 {
                AstType::Set { inner: Box::new(args.into_iter().next().unwrap()), span: self.span_from(start) }
            } else if name == "queue" && args.len() == 1 {
                AstType::Queue { inner: Box::new(args.into_iter().next().unwrap()), span: self.span_from(start) }
            } else if name == "stack" && args.len() == 1 {
                AstType::Stack { inner: Box::new(args.into_iter().next().unwrap()), span: self.span_from(start) }
            } else if name == "deque" && args.len() == 1 {
                AstType::Deque { inner: Box::new(args.into_iter().next().unwrap()), span: self.span_from(start) }
            } else if name == "linked_list" && args.len() == 1 {
                AstType::LinkedList { inner: Box::new(args.into_iter().next().unwrap()), span: self.span_from(start) }
            } else if name == "chan" && args.len() == 1 {
                AstType::Chan { inner: Box::new(args.into_iter().next().unwrap()), span: self.span_from(start) }
            } else {
                AstType::Generic { name, args, span: self.span_from(start) }
            }
        } else {
            AstType::User { name, span: self.span_from(start) }
        };
        // Postfix `?` for optional types: T?, list<i32>?
        if self.at(TokenKind::Question) {
            self.advance();
            base = AstType::Optional { inner: Box::new(base), span: self.span_from(start) };
        }
        // Postfix `!` for error-returning types: T!, list<i32>!
        // (already uses AstType::Error which exists)
        if self.at(TokenKind::Bang) {
            self.advance();
            base = AstType::Error { inner: Box::new(base), span: self.span_from(start) };
        }
        Ok(base)
    }

    // -----------------------------------------------------------------------
    // Expression parsing
    // -----------------------------------------------------------------------
}
