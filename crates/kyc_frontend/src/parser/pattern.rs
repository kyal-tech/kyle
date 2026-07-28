use kyc_core::ast::*;
use kyc_core::span::Span;
use crate::token::{Token, TokenKind};

use super::Parser;

impl Parser {
    pub(crate) fn parse_pattern(&mut self) -> Result<Pattern, String> {
        let start = self.pos;
        // `is Type:` — type test pattern
        if self.at(TokenKind::Is) {
            self.advance();
            let type_ = self.parse_type()?;
            return Ok(Pattern::IsType { type_, span: self.span_from(start) });
        }
        // Check for ok(v) pattern (keyword)
        if self.at(TokenKind::OkKw) {
            let start = self.pos;
            self.advance();
            self.expect(TokenKind::LParen)?;
            let inner = self.parse_pattern()?;
            self.expect(TokenKind::RParen)?;
            return Ok(Pattern::EnumVariant {
                enum_name: "Result".to_string(), variant: "Ok".to_string(),
                args: vec![inner], span: self.span_from(start),
            });
        }
        if self.at_identifier() {
            let name = self.eat_identifier();
            // Check for some(v) pattern (shorthand for Option.Some(v))
            if name == "some" && self.at(TokenKind::LParen) {
                self.advance();
                let inner = self.parse_pattern()?;
                self.expect(TokenKind::RParen)?;
                return Ok(Pattern::EnumVariant {
                    enum_name: "Option".to_string(), variant: "Some".to_string(),
                    args: vec![inner], span: self.span_from(start),
                });
            }
            // none pattern (shorthand for Option.None)
            if name == "none" {
                return Ok(Pattern::EnumVariant {
                    enum_name: "Option".to_string(), variant: "None".to_string(),
                    args: vec![], span: self.span_from(start),
                });
            }
            // Check for error(e) pattern (identifier "error" followed by LParen)
            if name == "error" && self.at(TokenKind::LParen) {
                self.advance();
                let inner = self.parse_pattern()?;
                self.expect(TokenKind::RParen)?;
                return Ok(Pattern::EnumVariant {
                    enum_name: "Result".to_string(), variant: "Err".to_string(),
                    args: vec![inner], span: self.span_from(start),
                });
            }
            // Check for enum variant pattern: Option.Some(v)
            if self.at(TokenKind::Dot) {
                self.advance();
                let variant = self.eat_identifier();
                let args = if self.at(TokenKind::LParen) {
                    self.advance();
                    let mut patterns = Vec::new();
                    loop {
                        patterns.push(self.parse_pattern()?);
                        if self.at(TokenKind::Comma) { self.advance(); }
                        else { break; }
                    }
                    self.expect(TokenKind::RParen)?;
                    patterns
                } else {
                    vec![]
                };
                return Ok(Pattern::EnumVariant {
                    enum_name: name,
                    variant,
                    args,
                    span: self.span_from(start),
                });
            }
            return Ok(Pattern::Identifier { name, span: self.span_from(start) });
        }
        // Tuple pattern: (p1, p2, ...)
        if self.at(TokenKind::LParen) {
            self.advance();
            let mut elements = Vec::new();
            loop {
                elements.push(self.parse_pattern_or()?);
                if self.at(TokenKind::Comma) { self.advance(); }
                else { break; }
            }
            self.expect(TokenKind::RParen)?;
            return Ok(Pattern::Tuple { elements, span: self.span_from(start) });
        }
        let lit = match &self.current()?.kind {
            TokenKind::Integer(s) => {
                let n: i64 = s.parse().unwrap_or_else(|_| {
                    s.parse::<u64>().map(|u| u as i64).unwrap_or(0)
                });
                Literal::Integer(n)
            }
            TokenKind::Float(s) => {
                let n: f64 = s.parse().unwrap_or(0.0);
                Literal::Float(n)
            }
            TokenKind::String(s) => Literal::String(s.clone()),
            TokenKind::True => Literal::Boolean(true),
            TokenKind::False => Literal::Boolean(false),
            TokenKind::None => Literal::None,
            _ => {
                let found = format!("{:?}", self.current()?.kind);
                return Err(format!("expected pattern, found {}", found));
            }
        };
        self.advance();
        // Check for range patterns: 1..=5 or 1..<5
        if self.at(TokenKind::DotDotEquals) || self.at(TokenKind::DotDotLess) {
            let inclusive = self.at(TokenKind::DotDotEquals);
            self.advance();
            let end = match &self.current()?.kind {
                TokenKind::Integer(s) => { let n: i64 = s.parse().unwrap_or(0); self.advance(); Literal::Integer(n) }
                _ => { return Err("expected integer literal after range".to_string()); }
            };
            return Ok(Pattern::Range { start: lit, end, inclusive, span: self.span_from(start) });
        }
        Ok(Pattern::Literal {
            value: lit,
            span: self.span_from(start),
        })
    }

    /// Parse a pattern, possibly with `|` for or-patterns.
    pub(crate) fn parse_pattern_or(&mut self) -> Result<Pattern, String> {
        let start = self.pos;
        let mut patterns = vec![self.parse_pattern()?];
        while self.at(TokenKind::Pipe) {
            self.advance();
            patterns.push(self.parse_pattern()?);
        }
        if patterns.len() == 1 {
            Ok(patterns.into_iter().next().unwrap())
        } else {
            Ok(Pattern::Or { patterns, span: self.span_from(start) })
        }
    }

}
