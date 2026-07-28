// kyc_frontend::token — Token types for the KL lexer
//
// Reference: docs/02-formal-grammar.md
// Each token carries its source Span for error reporting.

use kyc_core::span::Span;

/// A single token produced by the lexer.
#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }

    /// Returns true if this token matches the given kind (ignoring payload).
    pub fn is(&self, kind: &TokenKind) -> bool {
        std::mem::discriminant(&self.kind) == std::mem::discriminant(kind)
    }
}

/// All token kinds recognized by the KL lexer.
#[derive(Clone, Debug, PartialEq)]
pub enum TokenKind {
    // -----------------------------------------------------------------------
    // Literals
    // -----------------------------------------------------------------------
    Identifier(String),
    Integer(String),
    Float(String),
    String(String),
    Char(String),
    Boolean(bool),

    // -----------------------------------------------------------------------
    // Keywords
    // -----------------------------------------------------------------------
    Fn,         // fn
    Class,      // class
    Abstract,   // abstract
    Final,      // final
    Struct,     // struct (temporary alias for final class)
    Enum,       // enum
    Contract,   // contract
    If,         // if
    Elif,       // elif
    Else,       // else
    While,      // while
    For,        // for
    In,         // in
    Match,      // match
    Return,     // return
    Break,      // break
    Continue,   // continue
    Defer,      // defer
    Guard,      // guard
    Unsafe,     // unsafe
    Async,      // async
    Await,      // await
    Const,      // const
    Loop,       // loop
    Type,       // type
    True,       // true
    False,      // false
    None,       // None
    Null,       // null
    OkKw,       // ok (keyword alias to avoid conflict with Result)
    Extern,     // extern
    Use,        // use
    Import,     // import
    From,       // from
    As,         // as
    Is,         // is (type test)
    Super,      // super (parent class reference)
    Get,        // get
    Set,        // set
    Static,     // static (for static methods)
    Implements, // implements
    // Note: Mut removed — use := (Walrus) instead

    // Attributes
    At,         // @
    Hash,       // # (for `#[attr]`)

    // Logical
    And,        // &&
    Or,         // ||

    // -----------------------------------------------------------------------
    // Operators
    // -----------------------------------------------------------------------
    Plus,           // +
    Minus,          // -
    Star,           // *
    Slash,          // /
    Percent,        // %
    StarStar,       // **
    PlusPercent,    // +%
    MinusPercent,   // -%
    StarPercent,    // *%
    Equals,         // =
    EqualsEquals,   // ==
    Bang,           // !
    BangEquals,     // !=
    Less,           // <
    Greater,        // >
    LessEquals,     // <=
    GreaterEquals,  // >=

    // Bitwise
    Ampersand,          // &
    Pipe,               // |
    Caret,              // ^
    Tilde,              // ~
    LessLess,           // <<
    GreaterGreater,     // >>
    AmpersandEquals,    // &=
    PipeEquals,         // |=
    CaretEquals,        // ^=
    LessLessEquals,     // <<=
    GreaterGreaterEquals, // >>=

    // Declaration & assignment
    Walrus,         // := (now used for constants, replaces ::=)

    // Compound assignment
    PlusEquals,     // +=
    MinusEquals,    // -=
    StarEquals,     // *=
    SlashEquals,    // /=
    PercentEquals,  // %=

    // Delimiters & punctuation
    Dot,            // .
    DotDot,         // ..
    DotDotDot,      // ...
    DotDotEquals,   // ..=
    DotDotLess,     // ..<
    Comma,          // ,
    Colon,          // :
    Semicolon,      // ;
    Arrow,          // ->
    FatArrow,       // =>
    Question,       // ?
    QuestionDot,    // ?.
    QuestionQuestion, // ??

    // Brackets
    LParen,     // (
    RParen,     // )
    LBracket,   // [
    RBracket,   // ]
    LBrace,     // {
    RBrace,     // }

    // -----------------------------------------------------------------------
    // Special
    // -----------------------------------------------------------------------
    Indent,
    Dedent,
    Newline,
    Eof,
    LexError(String),
}
