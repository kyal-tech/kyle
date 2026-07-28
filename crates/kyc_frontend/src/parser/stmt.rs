use kyc_core::ast::*;
use kyc_core::span::Span;
use crate::token::{Token, TokenKind};

use super::Parser;

impl Parser {
    pub(crate) fn parse_block(&mut self) -> Result<Block, String> {
        let start = self.pos;
        let single_line = !self.at(TokenKind::Newline);
        if self.at(TokenKind::Newline) { self.advance(); }
        if self.at(TokenKind::Indent) { self.advance(); }
        let mut statements = Vec::new();
        let mut had_error = false;
        loop {
            if self.at(TokenKind::Dedent) { self.advance(); break; }
            if self.at(TokenKind::Eof) { break; }
            if self.at(TokenKind::Newline) { self.advance(); continue; }
            match self.parse_stmt() {
                Ok(stmt) => statements.push(stmt),
                Err(msg) => {
                    had_error = true;
                    self.error(msg);
                    if !single_line {
                        self.sync_to_stmt_boundary();
                    }
                }
            }
            if single_line { break; }
        }
        if had_error && !single_line {
            // After recovering within the block, return an empty block — the
            // caller will see the error but we've already consumed all tokens
            // up to the dedent/eof. Return the partially-parsed block anyway
            // so the outer level can continue.
            return Ok(Block { statements, span: self.span_from(start) });
        }
        if single_line {
            if had_error {
                // Single-line body: can't recover, propagate upward
                return Err(self.errors.join("\n"));
            }
            while self.at(TokenKind::Newline) {
                self.advance();
            }
        }
        Ok(Block { statements, span: self.span_from(start) })
    }

    pub(crate) fn parse_stmt(&mut self) -> Result<Stmt, String> {
        // Check for labeled loops: `label: for/while/loop`
        let label = self.peek_loop_label();
        if label.is_some() {
            self.advance(); // consume identifier
            self.advance(); // consume ':'
        }
        if self.at(TokenKind::If) { return self.parse_if(); }
        if self.at(TokenKind::While) { return self.parse_while(label); }
        if self.at(TokenKind::For) { return self.parse_for(label); }
        if self.at(TokenKind::Match) { return self.parse_match(); }
        if self.at(TokenKind::Loop) {
            let start = self.pos;
            self.advance();
            self.expect(TokenKind::Colon)?;
            let body = self.parse_block()?;
            return Ok(Stmt::Expression(Expr::Loop { body, label, span: self.span_from(start) }));
        }
        if self.at(TokenKind::Return) {
            self.advance();
            let expr = if self.current_is_expr_start() {
                Some(Box::new(self.parse_expr()?))
            } else { None };
            return Ok(Stmt::Return(expr));
        }
        if self.at(TokenKind::Break) {
            self.advance();
            let value = if self.at(TokenKind::At) {
                None
            } else if self.current_is_expr_start() {
                Some(Box::new(self.parse_expr()?))
            } else { None };
            let label = if self.at(TokenKind::At) {
                self.advance();
                Some(self.eat_identifier())
            } else { None };
            return Ok(Stmt::Break(value, label));
        }
        if self.at(TokenKind::Continue) {
            self.advance();
            let label = if self.at(TokenKind::At) {
                self.advance();
                Some(self.eat_identifier())
            } else { None };
            return Ok(Stmt::Continue(label));
        }
        if self.at(TokenKind::Defer) {
            let start = self.pos;
            self.advance();
            let call = self.parse_expr()?;
            return Ok(Stmt::Defer(DeferStmt { call: Box::new(call), span: self.span_from(start) }));
        }
        if self.at(TokenKind::Guard) {
            let start = self.pos;
            self.advance();
            let condition = self.parse_expr()?;
            self.expect_keyword("else")?;
            self.expect(TokenKind::Colon)?;
            let body = self.parse_block()?;
            return Ok(Stmt::Guard(GuardStmt { condition: Box::new(condition), body, span: self.span_from(start) }));
        }
        if self.at(TokenKind::Unsafe) {
            let start = self.pos;
            self.advance();
            self.expect(TokenKind::Colon)?;
            let body = self.parse_block()?;
            return Ok(Stmt::Unsafe(UnsafeBlock { body, span: self.span_from(start) }));
        }
        // Variable declaration: `ident [":" type] "=" expr` or `ident ":" "&" type "=" expr` (mutable)
        // Handle typed declarations first: `ident : type = expr`
        if self.at_identifier() && self.tokens.get(self.pos + 1).map_or(false, |t| t.is(&TokenKind::Colon)) {
            let start = self.pos;
            let name = self.eat_identifier();
            self.advance(); // ':'
            let type_ = self.parse_type()?;
            // Check if the type starts with `&` → mutable variable
            let is_mutable = matches!(type_, AstType::Mutable { .. });
            if self.at(TokenKind::Equals) {
                self.advance();
                let value = self.parse_expr()?;
                return Ok(Stmt::Variable(VariableDecl {
                    name, type_: Some(type_), value: Box::new(value), is_mutable, span: self.span_from(start),
                }));
            }
            return Err("expected '=' after typed declaration".to_string());
        }
        // Constant declaration: `NAME := expr` (unambiguous)
        if self.at_identifier() && self.tokens.get(self.pos + 1).map_or(false, |t| t.is(&TokenKind::Walrus)) {
            let start = self.pos;
            let name = self.eat_identifier();
            self.advance(); // consume :=
            let value = self.parse_expr()?;
            return Ok(Stmt::Variable(VariableDecl {
                name, type_: None, value: Box::new(value), is_mutable: false, span: self.span_from(start),
            }));
        }
        // Destructuring: `(x, y) = expr` or `(x, y) := expr`
        if self.peek_destructure() {
            return self.parse_destructure();
        }
        // Binding-if or assignment: `ident = expr [: block]`
        if self.at_identifier() && self.peek_equals() {
            let start = self.pos;
            let name = self.eat_identifier();
            self.advance(); // '='
            let value = self.parse_expr()?;
            if self.at(TokenKind::Colon) {
                self.advance();
                let body = self.parse_block()?;
                let else_branch = if self.at(TokenKind::Else) {
                    self.advance();
                    Some(self.parse_block()?)
                } else { None };
                Ok(Stmt::BindingIf(BindingIf {
                    name, value: Box::new(value), body, else_branch, span: self.span_from(start),
                }))
            } else {
                Ok(Stmt::Expression(Expr::Assignment {
                    target: Box::new(Expr::Identifier { name, span: self.span_from(start) }),
                    operator: None,
                    value: Box::new(value),
                    span: self.span_from(start),
                }))
            }
        } else {
            self.parse_expr_stmt()
        }
    }

    pub(crate) fn parse_if(&mut self) -> Result<Stmt, String> {
        let start = self.pos;
        self.advance();
        // `if name = expr:` — BindingIf
        if self.peek_equals() {
            return self.parse_binding_if();
        }
        let condition = self.parse_expr()?;
        self.expect(TokenKind::Colon)?;
        let body = self.parse_block()?;
        while self.at(TokenKind::Newline) { self.advance(); }
        let mut elif_branches = Vec::new();
        let mut else_branch = None;
        while self.at(TokenKind::Elif) {
            let elif_start = self.pos;
            self.advance();
            let cond = self.parse_expr()?;
            self.expect(TokenKind::Colon)?;
            let body = self.parse_block()?;
            while self.at(TokenKind::Newline) { self.advance(); }
            elif_branches.push(ElifBranch {
                condition: Box::new(cond),
                body,
                span: self.span_from(elif_start),
            });
        }
        if self.at(TokenKind::Else) {
            self.advance();
            self.expect(TokenKind::Colon)?;
            else_branch = Some(self.parse_block()?);
        }
        Ok(Stmt::If(IfStmt { condition: Box::new(condition), body, elif_branches, else_branch, span: self.span_from(start) }))
    }

    pub(crate) fn parse_binding_if(&mut self) -> Result<Stmt, String> {
        let start = self.pos;
        let name = self.eat_identifier();
        self.expect(TokenKind::Equals)?;
        let value = self.parse_expr()?;
        self.expect(TokenKind::Colon)?;
        let body = self.parse_block()?;
        let else_branch = if self.at(TokenKind::Else) {
            self.advance();
            self.expect(TokenKind::Colon)?;
            Some(self.parse_block()?)
        } else { None };
        Ok(Stmt::BindingIf(BindingIf {
            name, value: Box::new(value), body, else_branch, span: self.span_from(start),
        }))
    }

    pub(crate) fn parse_while(&mut self, label: Option<String>) -> Result<Stmt, String> {
        let start = self.pos;
        self.advance();
        let condition = self.parse_expr()?;
        self.expect(TokenKind::Colon)?;
        let body = self.parse_block()?;
        let else_branch = if self.at(TokenKind::Else) {
            self.advance();
            self.expect(TokenKind::Colon)?;
            Some(self.parse_block()?)
        } else { None };
        Ok(Stmt::While(WhileStmt { condition: Box::new(condition), body, else_branch, label, span: self.span_from(start) }))
    }

    pub(crate) fn parse_for(&mut self, label: Option<String>) -> Result<Stmt, String> {
        let start = self.pos;
        self.advance();
        let first = self.eat_identifier();
        let (variable, index_variable) = if self.at(TokenKind::Comma) {
            self.advance();
            let second = self.eat_identifier();
            (second, Some(first))
        } else {
            (first, None)
        };
        self.expect_keyword("in")?;
        let iterable = self.parse_expr()?;
        self.expect(TokenKind::Colon)?;
        let body = self.parse_block()?;
        let else_branch = if self.at(TokenKind::Else) {
            self.advance();
            self.expect(TokenKind::Colon)?;
            Some(self.parse_block()?)
        } else { None };
        Ok(Stmt::For(ForStmt { variable, index_variable, iterable: Box::new(iterable), body, else_branch, label, span: self.span_from(start) }))
    }

    pub(crate) fn parse_match(&mut self) -> Result<Stmt, String> {
        let start = self.pos;
        self.advance();
        let expression = self.parse_expr()?;
        self.expect(TokenKind::Colon)?;
        // Consume Newline and Indent that precede the match body
        if self.at(TokenKind::Newline) { self.advance(); }
        if self.at(TokenKind::Indent) { self.advance(); }
        let mut arms = Vec::new();
        loop {
            if self.at(TokenKind::Newline) { self.advance(); continue; }
            if self.at(TokenKind::Dedent) { self.advance(); break; }
            if self.at(TokenKind::Eof) { break; }
            let arm_start = self.pos;
            let pattern = self.parse_pattern_or()?;
            let guard = if self.at(TokenKind::If) {
                self.advance();
                Some(Box::new(self.parse_expr()?))
            } else { None };
            self.expect(TokenKind::Colon)?;
            let body = self.parse_block()?;
            arms.push(MatchArm { pattern, guard, body, span: self.span_from(arm_start) });
        }
        Ok(Stmt::Match(MatchStmt { expression: Box::new(expression), arms, span: self.span_from(start) }))
    }

    pub(crate) fn parse_match_expr(&mut self) -> Result<Expr, String> {
        let start = self.pos;
        self.advance(); // consume 'match'
        let expression = self.parse_expr()?;
        self.expect(TokenKind::Colon)?;
        // Consume Newline and Indent that precede the match body
        if self.at(TokenKind::Newline) { self.advance(); }
        if self.at(TokenKind::Indent) { self.advance(); }
        let mut arms = Vec::new();
        loop {
            if self.at(TokenKind::Newline) { self.advance(); continue; }
            if self.at(TokenKind::Dedent) { self.advance(); break; }
            if self.at(TokenKind::Eof) { break; }
            let arm_start = self.pos;
            let pattern = self.parse_pattern_or()?;
            let guard = if self.at(TokenKind::If) {
                self.advance();
                Some(Box::new(self.parse_expr()?))
            } else { None };
            self.expect(TokenKind::Colon)?;
            let body = self.parse_block()?;
            arms.push(MatchArm { pattern, guard, body, span: self.span_from(arm_start) });
        }
        Ok(Expr::MatchExpr { expression: Box::new(expression), arms, span: self.span_from(start) })
    }

    pub(crate) fn parse_if_expr(&mut self) -> Result<Expr, String> {
        let start = self.pos;
        self.advance(); // consume 'if'
        let condition = self.parse_expr()?;
        self.expect(TokenKind::Colon)?;
        // Consume optional Newline before the then-expression
        while self.at(TokenKind::Newline) { self.advance(); }
        let then_body = self.parse_expr()?;
        // Consume optional Newline between branches
        while self.at(TokenKind::Newline) { self.advance(); }
        let mut elif_branches = Vec::new();
        while self.at(TokenKind::Elif) {
            let elif_start = self.pos;
            self.advance();
            let cond = self.parse_expr()?;
            self.expect(TokenKind::Colon)?;
            while self.at(TokenKind::Newline) { self.advance(); }
            let body = self.parse_expr()?;
            while self.at(TokenKind::Newline) { self.advance(); }
            elif_branches.push(IfExprElif { condition: Box::new(cond), body: Box::new(body), span: self.span_from(elif_start) });
        }
        let else_branch = if self.at(TokenKind::Else) {
            self.advance();
            self.expect(TokenKind::Colon)?;
            while self.at(TokenKind::Newline) { self.advance(); }
            let body = self.parse_expr()?;
            Some(Box::new(body))
        } else {
            return Err("if-expression requires an else branch".to_string());
        };
        Ok(Expr::IfExpr {
            condition: Box::new(condition),
            then_body: Box::new(then_body),
            elif_branches,
            else_branch,
            span: self.span_from(start),
        })
    }
}
