use kyc_core::ast::*;
use kyc_core::span::Span;
use crate::token::{Token, TokenKind};
use super::Parser;

impl Parser {
    pub(crate) fn parse_expr(&mut self) -> Result<Expr, String> {
        self.parse_binary(0)
    }

    pub(crate) fn parse_binary(&mut self, min_prec: u8) -> Result<Expr, String> {
        let start = self.pos;
        let mut left = self.parse_unary()?;
        loop {
            // Check for '??' — default operator (expr ?? default)
            if self.at(TokenKind::QuestionQuestion) {
                const NULL_COALESCE_PREC: u8 = 3;
                if NULL_COALESCE_PREC < min_prec { break; }
                let start = self.pos;
                self.advance();
                let right = self.parse_binary(NULL_COALESCE_PREC + 1)?;
                left = Expr::NullCoalesce {
                    left: Box::new(left),
                    right: Box::new(right),
                    span: self.span_from(start),
                };
                continue;
            }
            // Check for '?' — BOTH ternary (cond ? then : else) and error-prop (expr?)
            if self.at(TokenKind::Question) {
                const TERNARY_PREC: u8 = 2;
                if TERNARY_PREC < min_prec { break; }
                let saved = self.pos;
                self.advance(); // consume '?'
                // Try ternary first: parse middle expression, check for ':'
                match self.parse_binary(0) {
                    Ok(middle) => {
                        if self.at(TokenKind::Colon) {
                            self.advance(); // consume ':'
                            let right = self.parse_binary(TERNARY_PREC)?;
                            left = Expr::Ternary {
                                cond: Box::new(left),
                                then_expr: Box::new(middle),
                                else_expr: Box::new(right),
                                span: self.span_from(start),
                            };
                            continue; // allow chaining: a ? b : c ? d : e
                        }
                        // Not ternary — fall through to error prop
                    }
                    Err(_) => {
                        // Not ternary — fall through to error prop
                    }
                }
                // Restore and handle as error propagation
                self.pos = saved;
                self.advance(); // consume '?'
                left = Expr::ErrorProp { expression: Box::new(left), span: self.span_from(start) };
                continue;
            }
            let op = match self.current_operator() {
                Some(op) => op,
                None => break,
            };
            let prec = self.operator_precedence(&op);
            if prec < min_prec { break; }
            self.advance();
            let right = self.parse_binary(prec + 1)?;
            left = Expr::Binary {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
                span: self.span_from(start),
            };
        }
        Ok(left)
    }

    pub(crate) fn parse_unary(&mut self) -> Result<Expr, String> {
        if self.at(TokenKind::Minus) {
            let start = self.pos;
            self.advance();
            return Ok(Expr::Unary { operator: UnaryOp::Neg, operand: Box::new(self.parse_primary()?), span: self.span_from(start) });
        }
        if self.at(TokenKind::Bang) {
            let start = self.pos;
            self.advance();
            return Ok(Expr::Unary { operator: UnaryOp::Not, operand: Box::new(self.parse_primary()?), span: self.span_from(start) });
        }
        if self.at(TokenKind::Tilde) {
            let start = self.pos;
            self.advance();
            return Ok(Expr::Unary { operator: UnaryOp::BitNot, operand: Box::new(self.parse_primary()?), span: self.span_from(start) });
        }
        if self.at(TokenKind::Await) {
            let start = self.pos;
            self.advance();
            return Ok(Expr::Await { expression: Box::new(self.parse_primary()?), span: self.span_from(start) });
        }
        // `&expr` — borrow reference at call site
        if self.at(TokenKind::Ampersand) {
            let start = self.pos;
            self.advance();
            return Ok(Expr::BorrowRef { expression: Box::new(self.parse_primary()?), mutable: false, span: self.span_from(start) });
        }
        // `^expr` — mutable prefix (only valid before &)
        if self.at(TokenKind::Caret) {
            let start = self.pos;
            self.advance();
            // `^&` — mutable borrow
            if self.at(TokenKind::Ampersand) {
                self.advance();
                return Ok(Expr::BorrowRef { expression: Box::new(self.parse_primary()?), mutable: true, span: self.span_from(start) });
            }
            return Err("`^` before expression requires `&` (use `^&x` for mutable borrow)".to_string());
        }
        self.parse_primary()
    }

    pub(crate) fn parse_primary(&mut self) -> Result<Expr, String> {
        let start = self.pos;
        let tok = self.current()?;
        let expr = match &tok.kind {
            TokenKind::Integer(s) => {
                let val = if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
                    i64::from_str_radix(hex, 16).unwrap_or(0)
                } else if let Some(bin) = s.strip_prefix("0b").or_else(|| s.strip_prefix("0B")) {
                    i64::from_str_radix(bin, 2).unwrap_or(0)
                } else if let Some(oct) = s.strip_prefix("0o").or_else(|| s.strip_prefix("0O")) {
                    i64::from_str_radix(oct, 8).unwrap_or(0)
                } else {
                    // Try i64 first, fall back to u64 → two's complement i64
                    s.parse::<i64>().unwrap_or_else(|_| {
                        s.parse::<u64>().map(|u| u as i64).unwrap_or(0)
                    })
                };
                self.advance();
                Expr::Literal { value: Literal::Integer(val), span: self.span_from(start) }
            }
            TokenKind::Float(s) => {
                let val = s.parse::<f64>().unwrap_or(0.0);
                self.advance();
                Expr::Literal { value: Literal::Float(val), span: self.span_from(start) }
            }
            TokenKind::String(s) => {
                let val = s.clone();
                self.advance();
                if val.contains('{') {
                    self.parse_string_interp(&val, start)?
                } else {
                    Expr::Literal { value: Literal::String(val), span: self.span_from(start) }
                }
            }
            TokenKind::True => {
                self.advance();
                Expr::Literal { value: Literal::Boolean(true), span: self.span_from(start) }
            }
            TokenKind::False => {
                self.advance();
                Expr::Literal { value: Literal::Boolean(false), span: self.span_from(start) }
            }
            TokenKind::None => {
                self.advance();
                Expr::Literal { value: Literal::None, span: self.span_from(start) }
            }
            TokenKind::Null => {
                self.advance();
                Expr::Literal { value: Literal::Null, span: self.span_from(start) }
            }
            TokenKind::Char(s) => {
                let val = s.chars().next().unwrap_or('\0') as u8 as i32;
                self.advance();
                Expr::Literal { value: Literal::Char(val), span: self.span_from(start) }
            }
            TokenKind::Identifier(name) => {
                let val = name.clone();
                self.advance();
                Expr::Identifier { name: val, span: self.span_from(start) }
            }
            TokenKind::Set => {
                self.advance();
                Expr::Identifier { name: "set".to_string(), span: self.span_from(start) }
            }
            TokenKind::Type => {
                self.advance();
                Expr::Identifier { name: "type".to_string(), span: self.span_from(start) }
            }
            TokenKind::OkKw => {
                self.advance();
                Expr::Identifier { name: "ok".to_string(), span: self.span_from(start) }
            }
            TokenKind::LParen => {
                let start = self.pos;
                self.advance(); // consume '('
                // Try closure: (params) => expr or (params) RetType: body
                let saved = self.pos;
                let params = self.parse_closure_params();
                if self.at(TokenKind::RParen) {
                    self.advance(); // consume ')'
                    let result = if self.at(TokenKind::FatArrow) {
                        self.advance(); // consume '=>'
                        let body = self.parse_expr()?;
                        Expr::Closure { params, body: Box::new(body), span: self.span_from(start) }
                    } else if self.at(TokenKind::Colon) {
                        // Multi-line closure: (params): \n    body
                        self.advance(); // consume ':'
                        let body = self.parse_block()?;
                        let mut body_expr = Expr::Literal { value: Literal::None, span: self.span_from(start) };
                        for stmt in body.statements {
                            if let Stmt::Expression(e) = stmt {
                                body_expr = e;
                            }
                        }
                        Expr::Closure { params, body: Box::new(body_expr), span: self.span_from(start) }
                    } else if self.at_identifier() {
                        // Closure with return type: (params) RetType: body
                        let _saved2 = self.pos;
                        let _ret_type = self.eat_identifier();
                        if self.at(TokenKind::Colon) {
                            self.advance(); // consume ':'
                            // Parse same-line body only
                            let body = self.parse_expr()?;
                            Expr::Closure { params, body: Box::new(body), span: self.span_from(start) }
                        } else {
                            self.pos = saved;
                            self.parse_tuple_or_paren_expr(start)?
                        }
                    } else {
                        self.pos = saved;
                        self.parse_tuple_or_paren_expr(start)?
                    };
                    result
                } else {
                    // Not a closure — try tuple or parenthesized expr
                    self.pos = saved;
                    self.parse_tuple_or_paren_expr(start)?
                }
            }
            TokenKind::LBracket => {
                self.advance();
                while self.at(TokenKind::Newline) || self.at(TokenKind::Indent) || self.at(TokenKind::Dedent) {
                    self.advance();
                }
                if self.at(TokenKind::RBracket) {
                    self.advance();
                    Expr::List { elements: vec![], span: self.span_from(start) }
                } else {
                    let mut elements = Vec::new();
                    loop {
                        while self.at(TokenKind::Newline) || self.at(TokenKind::Indent) || self.at(TokenKind::Dedent) {
                            self.advance();
                        }
                        if self.at(TokenKind::RBracket) { break; }
                        if self.at(TokenKind::DotDotDot) {
                            let span_start = self.pos;
                            self.advance();
                            let expr = self.parse_expr()?;
                            elements.push(Expr::Spread { expression: Box::new(expr), span: self.span_from(span_start) });
                        } else {
                            elements.push(self.parse_expr()?);
                        }
                        if !self.at(TokenKind::Comma) { break; }
                        self.advance();
                    }
                    if elements.len() == 1 && self.at(TokenKind::Semicolon) {
                        self.advance();
                        let count = self.parse_expr()?;
                        self.expect(TokenKind::RBracket)?;
                        return Ok(Expr::ArrayRepeat { value: Box::new(elements.remove(0)), count: Box::new(count), span: self.span_from(start) });
                    }
                    self.expect(TokenKind::RBracket)?;
                    Expr::List { elements, span: self.span_from(start) }
                }
            }
            TokenKind::LBrace => {
                let start = self.pos;
                self.advance();
                while self.at(TokenKind::Newline) || self.at(TokenKind::Indent) || self.at(TokenKind::Dedent) {
                    self.advance();
                }
                // {} → dict vacío (backward compatible)
                if self.at(TokenKind::RBrace) {
                    self.advance();
                    Expr::Dictionary { entries: vec![], span: self.span_from(start) }
                } else {
                    // Lookahead: después del primer elemento, ¿hay : o ,/}?
                    let saved = self.pos;
                    let _first = self.parse_expr()?;
                    if self.at(TokenKind::Colon) {
                        // {key: val, ...} → DICT
                        self.pos = saved;
                        let mut entries = Vec::new();
                        while self.at(TokenKind::Newline) || self.at(TokenKind::Indent) || self.at(TokenKind::Dedent) {
                            self.advance();
                        }
                        loop {
                            if self.at(TokenKind::RBrace) { break; }
                            let key = if let Ok(tok) = self.current() {
                                match &tok.kind {
                                    TokenKind::String(s) => { let val = s.clone(); self.advance(); val }
                                    _ => self.eat_identifier()
                                }
                            } else { String::new() };
                            self.expect(TokenKind::Colon)?;
                            let value = self.parse_expr()?;
                            entries.push((key, value));
                            if self.at(TokenKind::Comma) {
                                self.advance();
                                while self.at(TokenKind::Newline) || self.at(TokenKind::Indent) || self.at(TokenKind::Dedent) {
                                    self.advance();
                                }
                            }
                            else { break; }
                        }
                        self.expect(TokenKind::RBrace)?;
                        Expr::Dictionary { entries, span: self.span_from(start) }
                    } else {
                        // {val, val, ...} → LIST
                        self.pos = saved;
                        while self.at(TokenKind::Newline) || self.at(TokenKind::Indent) || self.at(TokenKind::Dedent) {
                            self.advance();
                        }
                        let mut elements = vec![self.parse_expr()?];
                        while self.at(TokenKind::Comma) {
                            self.advance();
                            while self.at(TokenKind::Newline) || self.at(TokenKind::Indent) || self.at(TokenKind::Dedent) {
                                self.advance();
                            }
                            if self.at(TokenKind::RBrace) { break; }
                            elements.push(self.parse_expr()?);
                        }
                        self.expect(TokenKind::RBrace)?;
                        Expr::SetLiteral { collection_name: "set".to_string(), elements, span: self.span_from(start) }
                    }
                }
            }
            TokenKind::Async => {
                self.advance();
                // Check for async: block syntax
                if self.at(TokenKind::Colon) {
                    self.advance(); // consume ':'
                    let body = self.parse_block()?;
                    Expr::AsyncBlock { body, span: self.span_from(start) }
                } else if self.at(TokenKind::Newline) {
                    let body = self.parse_block()?;
                    Expr::AsyncBlock { body, span: self.span_from(start) }
                } else {
                    let expr = self.parse_expr()?;
                    Expr::Async { expression: Box::new(expr), span: self.span_from(start) }
                }
            }
            TokenKind::Match => {
                return self.parse_match_expr();
            }
            TokenKind::If => {
                return self.parse_if_expr();
            }
            TokenKind::Super => {
                self.advance();
                // V1: super is an alias for this (parent resolution not yet implemented)
                Expr::Identifier { name: "this".to_string(), span: self.span_from(start) }
            }
            _ => return Err(format!("unexpected token in expression: {:?}", tok.kind)),
        };
        self.parse_postfix(expr)
    }

    pub(crate) fn parse_postfix(&mut self, mut expr: Expr) -> Result<Expr, String> {
        loop {
            // Generic type args on identifiers or methods: Name<T>(args), obj.method<T>(args)
            if self.at(TokenKind::Less) && matches!(&expr, Expr::Identifier { .. } | Expr::PropertyAccess { .. }) {
                let saved = self.pos;
                self.advance();
                if let Ok(first_arg) = self.parse_type() {
                    let mut type_args = vec![first_arg];
                    while self.at(TokenKind::Comma) {
                        self.advance();
                        if let Ok(t) = self.parse_type() { type_args.push(t); } else { break; }
                    }
                    if self.at(TokenKind::Greater) {
                        self.advance();
                        let start = self.pos;
                        if self.at(TokenKind::LParen) {
                            self.advance();
                            let mut arguments = Vec::new();
                            while !self.at(TokenKind::RParen) && !self.at(TokenKind::Eof) {
                                arguments.push(self.parse_expr()?);
                                if self.at(TokenKind::Comma) { self.advance(); }
                            }
                            self.expect(TokenKind::RParen)?;
                            expr = Expr::FunctionCall {
                                target: Box::new(expr), arguments, type_args, span: self.span_from(start),
                            };
                            continue;
                        }
                        if self.at(TokenKind::LBrace) {
                            self.advance();
                            let mut fields = Vec::new();
                            while !self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof) {
                                let key = self.eat_identifier();
                                self.expect(TokenKind::Colon)?;
                                let value = self.parse_expr()?;
                                fields.push((key, value));
                                if self.at(TokenKind::Comma) { self.advance(); }
                            }
                            self.expect(TokenKind::RBrace)?;
                            expr = Expr::StructLiteral {
                                struct_name: if let Expr::Identifier { name, .. } = &expr { name.clone() } else { String::new() },
                                type_args, fields, span: self.span_from(start),
                            };
                            continue;
                        }
                        // After <T> without ( or { — push back and let normal postfix handle
                        self.pos = saved;
                    } else if self.at(TokenKind::GreaterGreater) {
                        self.advance();
                        let start = self.pos;
                        if self.at(TokenKind::LParen) {
                            // Function call with type args: identity<i32>(42)
                            self.advance();
                            let mut arguments = Vec::new();
                            while !self.at(TokenKind::RParen) && !self.at(TokenKind::Eof) {
                                arguments.push(self.parse_expr()?);
                                if self.at(TokenKind::Comma) { self.advance(); }
                            }
                            self.expect(TokenKind::RParen)?;
                            expr = Expr::FunctionCall {
                                target: Box::new(expr), arguments, type_args, span: self.span_from(start),
                            };
                        } else if self.at(TokenKind::LBrace) {
                            // Struct literal with type args: Container<i32>{ ... }
                            self.advance();
                            let mut fields = Vec::new();
                            while !self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof) {
                                let key = self.eat_identifier();
                                self.expect(TokenKind::Colon)?;
                                let value = self.parse_expr()?;
                                fields.push((key, value));
                                if self.at(TokenKind::Comma) { self.advance(); }
                            }
                            self.expect(TokenKind::RBrace)?;
                            expr = Expr::StructLiteral {
                                struct_name: if let Expr::Identifier { name, .. } = &expr { name.clone() } else { String::new() },
                                type_args, fields, span: self.span_from(start),
                            };
                        } else {
                            // Not followed by ( or { — backtrack
                            self.pos = saved;
                        }
                        continue; // handled, go to next postfix iteration
                    }
                }
                self.pos = saved;
            }
            if self.at(TokenKind::LParen) {
                let start = self.pos;
                self.advance();
                let mut arguments = Vec::new();
                while !self.at(TokenKind::RParen) && !self.at(TokenKind::Eof) {
                    arguments.push(self.parse_expr()?);
                    if self.at(TokenKind::Comma) { self.advance(); }
                }
                self.expect(TokenKind::RParen)?;
                expr = Expr::FunctionCall { target: Box::new(expr), arguments, type_args: vec![], span: self.span_from(start) };
            } else if self.at(TokenKind::Dot) {
                let start = self.pos;
                self.advance();
                // Tuple index access: .0, .1, .2 (digit after dot) → PropertyAccess {"0"}
                if self.current().map_or(false, |t| matches!(t.kind, TokenKind::Integer(_))) {
                    if let Ok(tok) = self.current() {
                        if let TokenKind::Integer(s) = &tok.kind {
                            let property = s.clone();
                            self.advance();
                            expr = Expr::PropertyAccess { object: Box::new(expr), property, span: self.span_from(start) };
                            continue;
                        }
                    }
                }
                let property = self.eat_identifier();
                expr = Expr::PropertyAccess { object: Box::new(expr), property, span: self.span_from(start) };
            } else if self.at(TokenKind::QuestionDot) {
                let start = self.pos;
                self.advance();
                let property = self.eat_identifier();
                expr = Expr::OptionalChain { target: Box::new(expr), property, span: self.span_from(start) };
            } else if self.at(TokenKind::LBracket) {
                let start = self.pos;
                self.advance();
                // Handle range slice forms: [..], [start..], [..end], [start..end]
                if self.at(TokenKind::DotDot) {
                    // [..] or [..end]
                    self.advance();
                    let end = if self.at(TokenKind::RBracket) {
                        None
                    } else {
                        Some(Box::new(self.parse_expr()?))
                    };
                    self.expect(TokenKind::RBracket)?;
                    expr = Expr::RangeSlice {
                        target: Box::new(expr),
                        start: None,
                        end,
                        span: self.span_from(start),
                    };
                } else {
                    let index = self.parse_expr()?;
                    // Check for [start..] or [start..end]
                    if self.at(TokenKind::DotDot) {
                        self.advance();
                        let end = if self.at(TokenKind::RBracket) {
                            None
                        } else {
                            Some(Box::new(self.parse_expr()?))
                        };
                        self.expect(TokenKind::RBracket)?;
                        expr = Expr::RangeSlice {
                            target: Box::new(expr),
                            start: Some(Box::new(index)),
                            end,
                            span: self.span_from(start),
                        };
                    } else {
                        self.expect(TokenKind::RBracket)?;
                        // Detect range expression inside brackets: items[start..end] → RangeSlice
                        if let Expr::Binary { left, operator: BinaryOp::Range, right, .. } = &index {
                            expr = Expr::RangeSlice {
                                target: Box::new(expr),
                                start: Some(left.clone()),
                                end: Some(right.clone()),
                                span: self.span_from(start),
                            };
                        } else if let Expr::Binary { left, operator: BinaryOp::RangeInclusive, right, .. } = &index {
                            expr = Expr::RangeSlice {
                                target: Box::new(expr),
                                start: Some(left.clone()),
                                end: Some(right.clone()),
                                span: self.span_from(start),
                            };
                        } else if let Expr::Binary { left, operator: BinaryOp::RangeExclusive, right, .. } = &index {
                            expr = Expr::RangeSlice {
                                target: Box::new(expr),
                                start: Some(left.clone()),
                                end: Some(right.clone()),
                                span: self.span_from(start),
                            };
                        } else {
                            expr = Expr::Index { target: Box::new(expr), index: Box::new(index), span: self.span_from(start) };
                        }
                    }
                }
            } else if self.at(TokenKind::LBrace) {
                // Struct literal (no generics): Identifier { field: value, ... }
                // Also: set{1, 2, 3} — function call with brace syntax
                let start = self.pos;
                if let Expr::Identifier { name: struct_name, .. } = &expr {
                    let sname = struct_name.clone();
                    self.advance(); // consume '{'
                    // Check if this is a set{...} literal (no key:value pairs)
                    let is_set_literal = sname == "set" || sname == "queue" || sname == "stack"
                        || sname == "deque" || sname == "linked_list";
                    if is_set_literal {
                        // Parse as collection literal: set{1, 2, 3} / queue{1, 2} etc
                        let mut elements = Vec::new();
                        while !self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof) {
                            while self.at(TokenKind::Newline) || self.at(TokenKind::Indent) || self.at(TokenKind::Dedent) {
                                self.advance();
                            }
                            if self.at(TokenKind::RBrace) { break; }
                            elements.push(self.parse_expr()?);
                            if self.at(TokenKind::Comma) { self.advance(); }
                        }
                        self.expect(TokenKind::RBrace)?;
                        expr = Expr::SetLiteral { collection_name: sname.clone(), elements, span: self.span_from(start) };
                    } else {
                        // Normal struct literal with field: value pairs
                        let mut fields = Vec::new();
                        while !self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof) {
                            // Skip newlines and indentation between fields
                            while self.at(TokenKind::Newline) || self.at(TokenKind::Indent) || self.at(TokenKind::Dedent) {
                                self.advance();
                            }
                            if self.at(TokenKind::RBrace) { break; }
                            let key = self.eat_identifier();
                            if key.is_empty() { break; }
                            self.expect(TokenKind::Colon)?;
                            let value = self.parse_expr()?;
                            fields.push((key, value));
                            if self.at(TokenKind::Comma) { self.advance(); }
                        }
                        self.expect(TokenKind::RBrace)?;
                        expr = Expr::StructLiteral { struct_name: sname, type_args: vec![], fields, span: self.span_from(start) };
                    }
                } else {
                    break;
                }
            } else if self.at(TokenKind::Bang) {
                // Postfix `!` for error propagation: expr !
                let start = self.pos;
                self.advance();
                expr = Expr::ErrorProp { expression: Box::new(expr), span: self.span_from(start) };
            } else {
                break;
            }
        }
        Ok(expr)
    }

}
