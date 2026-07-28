// kyc_core::span — Source location tracking
//
// Every AST node carries a Span to enable precise error reporting.
// The Span points to the source file, line, and column range.

/// A position in source code (1-indexed line/column, 0-indexed byte offset).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}

/// A range in source code from `start` to `end`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
    pub start: Position,
    pub end: Position,
    pub file_id: usize,
}

impl Span {
    pub fn dummy() -> Self {
        Self {
            start: Position {
                line: 0,
                column: 0,
                offset: 0,
            },
            end: Position {
                line: 0,
                column: 0,
                offset: 0,
            },
            file_id: 0,
        }
    }
}
