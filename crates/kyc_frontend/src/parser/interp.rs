use kyc_core::ast::*;
use kyc_core::span::Span;
use crate::token::{Token, TokenKind};
use super::Parser;

impl Parser {
    pub(crate) fn parse_string_interp(&mut self, s: &str, start: usize) -> Result<Expr, String> {
        let mut parts: Vec<Expr> = Vec::new();
        let mut current = String::new();
        let chars: Vec<char> = s.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            if chars[i] == '\\' && i + 1 < chars.len() && (chars[i + 1] == '{' || chars[i + 1] == '}') {
                current.push(chars[i + 1]);
                i += 2;
            } else if chars[i] == '{' && i + 1 < chars.len() && chars[i + 1] != '"' {
                if !current.is_empty() {
                    parts.push(Expr::Literal {
                        value: Literal::String(std::mem::take(&mut current)),
                        span: self.span_from(start),
                    });
                }
                i += 1;
                let expr_start = i;
                let mut depth = 1;
                while i < chars.len() && depth > 0 {
                    if chars[i] == '{' { depth += 1; }
                    else if chars[i] == '}' { depth -= 1; }
                    if depth > 0 { i += 1; }
                }
                let expr_text: String = chars[expr_start..i].iter().collect();
                let expr = self.parse_interp_expr_text(&expr_text)?;
                parts.push(expr);
                i += 1;
            } else {
                current.push(chars[i]);
                i += 1;
            }
        }

        if !current.is_empty() {
            parts.push(Expr::Literal {
                value: Literal::String(current),
                span: self.span_from(start),
            });
        }

        if parts.is_empty() {
            parts.push(Expr::Literal {
                value: Literal::String(String::new()),
                span: self.span_from(start),
            });
        }

        // If only one part and it's a string literal, return a plain literal (no interpolation)
        if parts.len() == 1 {
            if let Expr::Literal { .. } = &parts[0] {
                return Ok(parts.remove(0));
            }
        }

        Ok(Expr::StringInterp { parts, span: self.span_from(start) })
    }

    /// Parse an expression from raw text (used inside `{...}` in string interpolation).
    pub(crate) fn parse_interp_expr_text(&self, text: &str) -> Result<Expr, String> {
        use crate::lexer::Lexer;
        let mut lexer = Lexer::new(text);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        parser.parse_expr()
    }
}
