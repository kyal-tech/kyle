// kyc_frontend::parser — Recursive descent parser for KL
//
// Transforms a token stream into an AST.
// Reference: docs/02-formal-grammar.md, docs/03-ast-specification.md

use kyc_core::ast::*;
use kyc_core::span::Span;
use crate::token::{Token, TokenKind};

/// The KL recursive-descent parser.
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    errors: Vec<String>,
    links: Vec<String>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0, errors: vec![], links: vec![] }
    }

    pub(crate) fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub(crate) fn error(&mut self, msg: String) {
        self.errors.push(msg);
    }

    /// Return true if the current token starts a statement.
    pub(crate) fn at_stmt_start(&self) -> bool {
        self.current().map_or(false, |t| matches!(t.kind,
            TokenKind::If | TokenKind::While | TokenKind::For | TokenKind::Match
            | TokenKind::Loop | TokenKind::Return | TokenKind::Break
            | TokenKind::Continue | TokenKind::Fn
        ))
    }

    /// Skip tokens until a synchronization point: dedent, eof, newline,
    /// or a keyword that begins a statement or declaration.
    pub(crate) fn sync_to_stmt_boundary(&mut self) {
        // Always advance past the token that caused the error first.
        self.advance();
        let mut safety = 0;
        while safety < 100 {
            safety += 1;
            if self.pos >= self.tokens.len() {
                return;
            }
            if self.at(TokenKind::Dedent) || self.at(TokenKind::Eof) {
                return;
            }
            if self.at(TokenKind::Newline) || self.at(TokenKind::Indent) {
                self.advance();
                return;
            }
            if self.at_stmt_start() {
                return;
            }
            self.advance();
        }
    }

    /// Build a span covering from `start_pos` (token index at start) to the
    /// last consumed token (self.pos - 1). If no tokens consumed, returns a
    /// span around the current token or a dummy span.
    pub(crate) fn span_from(&self, start_pos: usize) -> Span {
        let start = &self.tokens[start_pos].span.start;
        let end_pos = self.pos.saturating_sub(1).max(start_pos);
        let end = &self.tokens[end_pos].span.end;
        Span {
            start: *start,
            end: *end,
            file_id: 0,
        }
    }

    /// Parse the full token stream into a Program AST node.
    pub fn parse(&mut self) -> Result<Program, String> {
        let start = self.pos;
        let mut declarations = Vec::new();
        loop {
            if self.at(TokenKind::Eof) {
                break;
            }
            if self.at(TokenKind::Newline) || self.at(TokenKind::Indent) || self.at(TokenKind::Dedent) {
                self.advance();
                continue;
            }
            match self.parse_decl() {
                Ok(decl) => declarations.push(decl),
                Err(msg) => {
                    self.error(msg);
                    self.sync_to_stmt_boundary();
                }
            }
        }
        if self.has_errors() {
            Err(self.errors.join("\n"))
        } else {
            Ok(Program {
                declarations,
                links: std::mem::take(&mut self.links),
                span: self.span_from(start),
            })
        }
    }

    // -----------------------------------------------------------------------
    // Declaration parsing
    // -----------------------------------------------------------------------

}

impl Parser {
    pub(crate) fn parse_expr_stmt(&mut self) -> Result<Stmt, String> {
        let expr = self.parse_expr()?;
        Ok(Stmt::Expression(expr))
    }

    // -----------------------------------------------------------------------
    // Token inspection helpers
    // -----------------------------------------------------------------------

    /// Returns true if the current token has the given kind (by discriminant).
    pub(crate) fn at(&self, kind: TokenKind) -> bool {
        self.current().map_or(false, |t| t.is(&kind))
    }

    /// Returns true if the current token is an Identifier.
    pub(crate) fn at_identifier(&self) -> bool {
        self.current().map_or(false, |t| matches!(t.kind,
            TokenKind::Identifier(_)
            | TokenKind::Set
        ))
    }

    /// Returns true if the current and next token look like `ident =`.
    pub(crate) fn peek_equals(&self) -> bool {
        if !self.at_identifier() { return false; }
        self.tokens.get(self.pos + 1).map_or(false, |t| t.is(&TokenKind::Equals))
    }

    /// Returns true if the current position looks like a destructuring assignment:
    /// `(ident, ...) =|:= expr`
    pub(crate) fn peek_destructure(&self) -> bool {
        if !self.at(TokenKind::LParen) { return false; }
        if self.tokens.get(self.pos + 1).map_or(false, |t| matches!(&t.kind, TokenKind::Identifier(_)))
            && self.tokens.get(self.pos + 2).map_or(false, |t| t.is(&TokenKind::Comma))
        {
            let mut depth = 1;
            let mut i = self.pos + 1;
            while i < self.tokens.len() && depth > 0 {
                if self.tokens[i].is(&TokenKind::LParen) { depth += 1; }
                else if self.tokens[i].is(&TokenKind::RParen) { depth -= 1; }
                i += 1;
            }
            if depth == 0 && i < self.tokens.len() {
                let next = &self.tokens[i];
                return next.is(&TokenKind::Equals) || next.is(&TokenKind::Walrus);
            }
        }
        false
    }

    /// Parse destructuring: `(x, y) = expr` or `(x, y) := expr`
    pub(crate) fn parse_destructure(&mut self) -> Result<Stmt, String> {
        let start = self.pos;
        self.advance(); // '('
        let mut elements = Vec::new();
        loop {
            let name = self.eat_identifier();
            if name.is_empty() {
                return Err("expected identifier in destructuring pattern".to_string());
            }
            elements.push(Expr::Identifier { name, span: self.span_from(start) });
            if self.at(TokenKind::Comma) { self.advance(); }
            else { break; }
        }
        self.expect(TokenKind::RParen)?;
        let is_mutable = self.at(TokenKind::Walrus);
        if self.at(TokenKind::Equals) || is_mutable {
            self.advance();
            let value = self.parse_expr()?;
            Ok(Stmt::Expression(Expr::Assignment {
                target: Box::new(Expr::Tuple { elements, span: self.span_from(start) }),
                operator: None,
                value: Box::new(value),
                span: self.span_from(start),
            }))
        } else {
            Err("expected '=' or ':=' after destructuring pattern".to_string())
        }
    }

    /// Check if the current position has a label before a loop keyword.
    /// Returns the label name if found, or None.
    pub(crate) fn peek_loop_label(&self) -> Option<String> {
        // Pattern: `identifier : for`, `identifier : while`, or `identifier : loop`
        if !self.at_identifier() { return None; }
        if !self.tokens.get(self.pos + 1).map_or(false, |t| t.is(&TokenKind::Colon)) { return None; }
        let after_colon = self.tokens.get(self.pos + 2)?;
        let is_loop = after_colon.is(&TokenKind::For)
            || after_colon.is(&TokenKind::While)
            || after_colon.is(&TokenKind::Loop);
        if is_loop {
            if let Ok(tok) = self.current() {
                if let TokenKind::Identifier(name) = &tok.kind {
                    return Some(name.clone());
                }
            }
        }
        None
    }

    /// Returns true if the next token is `:=` (Walrus) — constant declaration.
    pub(crate) fn peek_walrus(&self) -> bool {
        self.tokens.get(self.pos).map_or(false, |t| t.is(&TokenKind::Walrus))
    }

    /// Returns the kind of declaration operator at pos+1: Equals or Walrus.
    pub(crate) fn peek_decl_op(&self) -> Option<TokenKind> {
        self.tokens.get(self.pos + 1).and_then(|t| {
            if t.is(&TokenKind::Equals) { Some(TokenKind::Equals) }
            else if t.is(&TokenKind::Walrus) { Some(TokenKind::Walrus) }
            else { None }
        })
    }

    /// Returns the current token, or an error.
    pub(crate) fn current(&self) -> Result<&Token, String> {
        self.tokens.get(self.pos).ok_or_else(|| "unexpected end of input".to_string())
    }

    /// Advance one token.
    pub(crate) fn advance(&mut self) {
        self.pos += 1;
    }

    /// Consume the current identifier and return its name.
    /// Also accepts soft keywords (get, set) and common value keywords (None, True, False) as identifiers.
    pub(crate) fn eat_identifier(&mut self) -> String {
        if let Ok(tok) = self.current() {
            let name = match &tok.kind {
                TokenKind::Identifier(n) => Some(n.clone()),
                TokenKind::Get => Some("get".to_string()),
                TokenKind::Set => Some("set".to_string()),
                TokenKind::None => Some("None".to_string()),
                TokenKind::True => Some("True".to_string()),
                TokenKind::False => Some("False".to_string()),
                TokenKind::Type => Some("type".to_string()),
                TokenKind::For => Some("for".to_string()),
                TokenKind::In => Some("in".to_string()),
                TokenKind::Is => Some("is".to_string()),
                TokenKind::As => Some("as".to_string()),
                _ => None,
            };
            if let Some(n) = name {
                self.advance();
                return n;
            }
        }
        String::new()
    }

    /// Try to parse closure parameter list: zero or more comma-separated identifiers.
    /// Supports optional type annotations: `name: Type`.
    /// Advancement stops BEFORE the closing `)` (if any).
    pub(crate) fn parse_closure_params(&mut self) -> Vec<(String, Option<AstType>)> {
        let mut params = Vec::new();
        if self.at(TokenKind::RParen) { return params; }
        loop {
            let name = self.eat_identifier();
            if name.is_empty() { break; }
            let typ = if self.at(TokenKind::Colon) {
                self.advance();
                let tname = self.eat_identifier();
                if tname.is_empty() { None }
                else { Some(AstType::User { name: tname, span: self.span_from(self.pos) }) }
            } else { None };
            params.push((name, typ));
            if self.at(TokenKind::Comma) { self.advance(); }
            else { break; }
        }
        params
    }

    /// Parse a tuple or parenthesized expression (after backtracking from closure).
    pub(crate) fn parse_tuple_or_paren_expr(&mut self, start: usize) -> Result<Expr, String> {
        let first = self.parse_expr()?;
        if self.at(TokenKind::Comma) {
            let mut elements = vec![first];
            while self.at(TokenKind::Comma) {
                self.advance();
                elements.push(self.parse_expr()?);
            }
            self.expect(TokenKind::RParen)?;
            Ok(Expr::Tuple { elements, span: self.span_from(start) })
        } else {
            self.expect(TokenKind::RParen)?;
            Ok(first)
        }
    }

    /// Expect a specific token kind; return error if not found.
    pub(crate) fn expect(&mut self, kind: TokenKind) -> Result<(), String> {
        if self.at(kind.clone()) {
            self.advance();
            Ok(())
        } else {
            let found = self.current().map(|t| format!("{:?}", t.kind)).unwrap_or_else(|_| "EOF".into());
            Err(format!("expected {:?}, found {}", kind, found))
        }
    }

    /// Expect a keyword by its string representation and consume the token.
    pub(crate) fn expect_keyword(&mut self, word: &str) -> Result<(), String> {
        let tok = self.current()?;
        // Map keyword strings to their token kinds.
        let expected = match word {
            "else" => TokenKind::Else,
            "in" => TokenKind::In,
            "as" => TokenKind::As,
            "import" => TokenKind::Import,
            _ => return Err(format!("unknown expected keyword '{}'", word)),
        };
        if tok.is(&expected) {
            self.advance();
            Ok(())
        } else {
            Err(format!("expected keyword '{}', found {:?}", word, tok.kind))
        }
    }

    pub(crate) fn current_operator(&self) -> Option<BinaryOp> {
        let kind = self.current().ok()?;
        match &kind.kind {
            TokenKind::Plus => Some(BinaryOp::Add),
            TokenKind::Minus => Some(BinaryOp::Sub),
            TokenKind::Star => Some(BinaryOp::Mul),
            TokenKind::Slash => Some(BinaryOp::Div),
            TokenKind::Percent => Some(BinaryOp::Rem),
            TokenKind::StarStar => Some(BinaryOp::Pow),
            TokenKind::EqualsEquals => Some(BinaryOp::Eq),
            TokenKind::BangEquals => Some(BinaryOp::Neq),
            TokenKind::Less => Some(BinaryOp::Lt),
            TokenKind::Greater => Some(BinaryOp::Gt),
            TokenKind::LessEquals => Some(BinaryOp::Le),
            TokenKind::GreaterEquals => Some(BinaryOp::Ge),
            TokenKind::Ampersand => Some(BinaryOp::BitAnd),
            TokenKind::Pipe => Some(BinaryOp::BitOr),
            TokenKind::Caret => Some(BinaryOp::BitXor),
            TokenKind::LessLess => Some(BinaryOp::Shl),
            TokenKind::GreaterGreater => Some(BinaryOp::Shr),
            TokenKind::PlusPercent => Some(BinaryOp::AddPercent),
            TokenKind::MinusPercent => Some(BinaryOp::SubPercent),
            TokenKind::StarPercent => Some(BinaryOp::MulPercent),
            TokenKind::Equals => Some(BinaryOp::Assign),
            TokenKind::PlusEquals => Some(BinaryOp::AddAssign),
            TokenKind::MinusEquals => Some(BinaryOp::SubAssign),
            TokenKind::StarEquals => Some(BinaryOp::MulAssign),
            TokenKind::SlashEquals => Some(BinaryOp::DivAssign),
            TokenKind::PercentEquals => Some(BinaryOp::RemAssign),
            TokenKind::AmpersandEquals => Some(BinaryOp::BitAndAssign),
            TokenKind::PipeEquals => Some(BinaryOp::BitOrAssign),
            TokenKind::CaretEquals => Some(BinaryOp::BitXorAssign),
            TokenKind::LessLessEquals => Some(BinaryOp::ShlAssign),
            TokenKind::GreaterGreaterEquals => Some(BinaryOp::ShrAssign),
            TokenKind::And => Some(BinaryOp::And),
            TokenKind::Or => Some(BinaryOp::Or),
            TokenKind::DotDot => Some(BinaryOp::Range),
            TokenKind::DotDotEquals => Some(BinaryOp::RangeInclusive),
            TokenKind::DotDotLess => Some(BinaryOp::RangeExclusive),
            TokenKind::Is => Some(BinaryOp::Is),
            TokenKind::As => Some(BinaryOp::As),
            _ => Option::None,
        }
    }

    pub(crate) fn operator_precedence(&self, op: &BinaryOp) -> u8 {
        match op {
            BinaryOp::Assign | BinaryOp::AddAssign | BinaryOp::SubAssign
            | BinaryOp::MulAssign | BinaryOp::DivAssign | BinaryOp::RemAssign
            | BinaryOp::BitAndAssign | BinaryOp::BitOrAssign | BinaryOp::BitXorAssign
            | BinaryOp::ShlAssign | BinaryOp::ShrAssign => 1,
            BinaryOp::Range | BinaryOp::RangeInclusive | BinaryOp::RangeExclusive => 2,
            BinaryOp::Or => 2,
            BinaryOp::And => 3,
            BinaryOp::Eq | BinaryOp::Neq => 4,
            BinaryOp::Lt | BinaryOp::Gt | BinaryOp::Le | BinaryOp::Ge | BinaryOp::Is => 5,
            BinaryOp::BitOr => 6,
            BinaryOp::BitXor => 7,
            BinaryOp::BitAnd => 8,
            BinaryOp::Shl | BinaryOp::Shr => 9,
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::AddPercent | BinaryOp::SubPercent => 10,
            BinaryOp::Mul | BinaryOp::Div | BinaryOp::Rem | BinaryOp::MulPercent => 11,
            BinaryOp::Pow => 12,
            BinaryOp::As => 13,
        }
    }

    pub(crate) fn current_is_expr_start(&self) -> bool {
        self.current().map_or(false, |t| match t.kind {
            TokenKind::Integer(_) | TokenKind::Float(_) | TokenKind::String(_)
            | TokenKind::True | TokenKind::False | TokenKind::Identifier(_)
            | TokenKind::LParen | TokenKind::LBracket | TokenKind::LBrace
            | TokenKind::Minus | TokenKind::Bang | TokenKind::Tilde
            | TokenKind::Async | TokenKind::Match | TokenKind::If
            | TokenKind::OkKw | TokenKind::None => true,
            _ => false,
        })
    }
}


pub mod decl;
pub mod type_parser;
pub mod expr;
pub mod stmt;
pub mod pattern;
pub mod interp;
pub mod tests;
