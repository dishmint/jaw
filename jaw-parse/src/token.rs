use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn merge(self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    // Brackets and punctuation
    LBracket,
    RBracket,
    LParen,
    RParen,
    Slash,
    EmDash,
    Colon,
    Equals,
    Comma,
    At,
    Hash,
    Question,
    Pipe,

    // Special markers (characters found inside [...])
    Tilde,
    Ampersand,
    Caret,
    Asterisk,
    Bang,
    GreaterThan,
    Plus,
    Minus,

    // Literals
    Number(u32),
    Identifier(String),
    Text(String),

    // Keywords
    In,

    // Structure
    Newline,
    Indent(usize),
    Eof,
}
