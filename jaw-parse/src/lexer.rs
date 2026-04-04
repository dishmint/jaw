use crate::token::{Span, Token, TokenKind};

pub struct Lexer {
    source: Vec<char>,
    pos: usize,
    byte_pos: usize,
    at_line_start: bool,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            pos: 0,
            byte_pos: 0,
            at_line_start: true,
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        while self.pos < self.source.len() {
            if self.at_line_start {
                let indent = self.consume_indent();
                if indent > 0 {
                    tokens.push(Token::new(
                        TokenKind::Indent(indent),
                        Span::new(self.byte_pos - indent, self.byte_pos),
                    ));
                }
                self.at_line_start = false;
            }

            if self.pos >= self.source.len() {
                break;
            }

            let ch = self.current();

            match ch {
                '\n' => {
                    let start = self.byte_pos;
                    self.advance();
                    tokens.push(Token::new(TokenKind::Newline, Span::new(start, self.byte_pos)));
                    self.at_line_start = true;
                }
                '\r' => {
                    let start = self.byte_pos;
                    self.advance();
                    if self.pos < self.source.len() && self.current() == '\n' {
                        self.advance();
                    }
                    tokens.push(Token::new(TokenKind::Newline, Span::new(start, self.byte_pos)));
                    self.at_line_start = true;
                }
                '[' => {
                    self.lex_bracket_expr(&mut tokens);
                }
                '/' => {
                    if self.is_function_start() {
                        let start = self.byte_pos;
                        self.advance();
                        tokens.push(Token::new(TokenKind::Slash, Span::new(start, self.byte_pos)));
                    } else {
                        self.lex_text(&mut tokens);
                    }
                }
                '#' => {
                    let start = self.byte_pos;
                    self.advance();
                    tokens.push(Token::new(TokenKind::Hash, Span::new(start, self.byte_pos)));
                }
                ':' => {
                    let start = self.byte_pos;
                    self.advance();
                    tokens.push(Token::new(TokenKind::Colon, Span::new(start, self.byte_pos)));
                }
                '=' => {
                    let start = self.byte_pos;
                    self.advance();
                    tokens.push(Token::new(TokenKind::Equals, Span::new(start, self.byte_pos)));
                }
                ',' => {
                    let start = self.byte_pos;
                    self.advance();
                    tokens.push(Token::new(TokenKind::Comma, Span::new(start, self.byte_pos)));
                }
                '(' => {
                    let start = self.byte_pos;
                    self.advance();
                    tokens.push(Token::new(TokenKind::LParen, Span::new(start, self.byte_pos)));
                }
                ')' => {
                    let start = self.byte_pos;
                    self.advance();
                    tokens.push(Token::new(TokenKind::RParen, Span::new(start, self.byte_pos)));
                }
                '?' => {
                    let start = self.byte_pos;
                    self.advance();
                    tokens.push(Token::new(TokenKind::Question, Span::new(start, self.byte_pos)));
                }
                '|' => {
                    let start = self.byte_pos;
                    self.advance();
                    tokens.push(Token::new(TokenKind::Pipe, Span::new(start, self.byte_pos)));
                }
                '@' => {
                    let start = self.byte_pos;
                    self.advance();
                    tokens.push(Token::new(TokenKind::At, Span::new(start, self.byte_pos)));
                }
                '\u{2014}' => {
                    // Em dash (—)
                    let start = self.byte_pos;
                    self.advance();
                    tokens.push(Token::new(TokenKind::EmDash, Span::new(start, self.byte_pos)));
                }
                ' ' | '\t' => {
                    // Skip non-leading whitespace
                    self.advance();
                }
                _ => {
                    if ch.is_ascii_digit() {
                        self.lex_number(&mut tokens);
                    } else if is_ident_start(ch) {
                        self.lex_identifier(&mut tokens);
                    } else {
                        self.lex_text(&mut tokens);
                    }
                }
            }
        }

        tokens.push(Token::new(TokenKind::Eof, Span::new(self.byte_pos, self.byte_pos)));
        tokens
    }

    fn lex_bracket_expr(&mut self, tokens: &mut Vec<Token>) {
        let start = self.byte_pos;
        self.advance(); // consume '['

        // Skip whitespace inside bracket
        self.skip_spaces();

        if self.pos >= self.source.len() {
            tokens.push(Token::new(TokenKind::LBracket, Span::new(start, self.byte_pos)));
            return;
        }

        let inner = self.current();

        // Check for special single-char markers: ~ & ^ * ! > + -
        match inner {
            '~' | '&' | '^' | '*' | '!' | '>' | '+' | '-' => {
                // Peek ahead to see if this is [marker]
                let saved_pos = self.pos;
                let saved_byte = self.byte_pos;
                self.advance(); // consume marker char
                self.skip_spaces();

                if self.pos < self.source.len() && self.current() == ']' {
                    // It's a special marker bracket
                    tokens.push(Token::new(TokenKind::LBracket, Span::new(start, start + 1)));
                    let marker_kind = match inner {
                        '~' => TokenKind::Tilde,
                        '&' => TokenKind::Ampersand,
                        '^' => TokenKind::Caret,
                        '*' => TokenKind::Asterisk,
                        '!' => TokenKind::Bang,
                        '>' => TokenKind::GreaterThan,
                        '+' => TokenKind::Plus,
                        '-' => TokenKind::Minus,
                        _ => unreachable!(),
                    };
                    tokens.push(Token::new(marker_kind, Span::new(saved_byte, saved_byte + inner.len_utf8())));
                    let rb_start = self.byte_pos;
                    self.advance(); // consume ']'
                    tokens.push(Token::new(TokenKind::RBracket, Span::new(rb_start, self.byte_pos)));
                    return;
                } else {
                    // Not a marker, restore and treat as normal bracket
                    self.pos = saved_pos;
                    self.byte_pos = saved_byte;
                }
            }
            _ => {}
        }

        // Check for number: [123]
        if self.current().is_ascii_digit() {
            let saved_pos = self.pos;
            let saved_byte = self.byte_pos;
            let num_start = self.byte_pos;
            let mut num_val: u32 = 0;

            while self.pos < self.source.len() && self.current().is_ascii_digit() {
                num_val = num_val * 10 + self.current().to_digit(10).unwrap();
                self.advance();
            }
            let num_end = self.byte_pos;

            self.skip_spaces();

            if self.pos < self.source.len() && self.current() == ']' {
                tokens.push(Token::new(TokenKind::LBracket, Span::new(start, start + 1)));
                tokens.push(Token::new(TokenKind::Number(num_val), Span::new(num_start, num_end)));
                let rb_start = self.byte_pos;
                self.advance();
                tokens.push(Token::new(TokenKind::RBracket, Span::new(rb_start, self.byte_pos)));
                return;
            } else {
                self.pos = saved_pos;
                self.byte_pos = saved_byte;
            }
        }

        // Check for identifier: [VarName]
        if self.pos < self.source.len() && is_ident_start(self.current()) {
            let saved_pos = self.pos;
            let saved_byte = self.byte_pos;
            let id_start = self.byte_pos;
            let mut ident = String::new();

            while self.pos < self.source.len() && is_ident_char(self.current()) {
                ident.push(self.current());
                self.advance();
            }
            let id_end = self.byte_pos;

            self.skip_spaces();

            if self.pos < self.source.len() && self.current() == ']' {
                tokens.push(Token::new(TokenKind::LBracket, Span::new(start, start + 1)));
                tokens.push(Token::new(TokenKind::Identifier(ident), Span::new(id_start, id_end)));
                let rb_start = self.byte_pos;
                self.advance();
                tokens.push(Token::new(TokenKind::RBracket, Span::new(rb_start, self.byte_pos)));
                return;
            } else {
                self.pos = saved_pos;
                self.byte_pos = saved_byte;
            }
        }

        // Fallback: just emit LBracket
        tokens.push(Token::new(TokenKind::LBracket, Span::new(start, start + 1)));
    }

    fn lex_number(&mut self, tokens: &mut Vec<Token>) {
        let start = self.byte_pos;
        let mut val: u32 = 0;

        while self.pos < self.source.len() && self.current().is_ascii_digit() {
            val = val * 10 + self.current().to_digit(10).unwrap();
            self.advance();
        }

        tokens.push(Token::new(TokenKind::Number(val), Span::new(start, self.byte_pos)));
    }

    fn lex_identifier(&mut self, tokens: &mut Vec<Token>) {
        let start = self.byte_pos;
        let mut ident = String::new();

        while self.pos < self.source.len() && is_ident_char(self.current()) {
            ident.push(self.current());
            self.advance();
        }

        let kind = if ident == "in" {
            TokenKind::In
        } else {
            TokenKind::Identifier(ident)
        };

        tokens.push(Token::new(kind, Span::new(start, self.byte_pos)));
    }

    fn lex_text(&mut self, tokens: &mut Vec<Token>) {
        let start = self.byte_pos;
        let mut text = String::new();

        while self.pos < self.source.len() {
            let ch = self.current();
            if ch == '\n' || ch == '\r' || ch == '[' || ch == '?' || ch == '|'
                || ch == '#' || ch == '@' || ch == '(' || ch == ')' || ch == ','
                || ch == ':' || ch == '=' || ch == '\u{2014}'
            {
                break;
            }
            if ch == '/' && self.is_function_start() {
                break;
            }
            if is_ident_start(ch) {
                // Check if this is the start of an identifier we should lex separately
                break;
            }
            text.push(ch);
            self.advance();
        }

        if !text.is_empty() {
            tokens.push(Token::new(TokenKind::Text(text), Span::new(start, self.byte_pos)));
        }
    }

    fn consume_indent(&mut self) -> usize {
        let mut indent = 0;
        while self.pos < self.source.len() {
            match self.current() {
                '\t' => {
                    indent += 1;
                    self.advance();
                }
                ' ' => {
                    // Count spaces, 4 spaces = 1 indent level (configurable)
                    let mut spaces = 0;
                    while self.pos < self.source.len() && self.current() == ' ' {
                        spaces += 1;
                        self.advance();
                    }
                    indent += spaces / 4;
                    break;
                }
                _ => break,
            }
        }
        indent
    }

    fn skip_spaces(&mut self) {
        while self.pos < self.source.len() && (self.current() == ' ' || self.current() == '\t') {
            self.advance();
        }
    }

    fn is_function_start(&self) -> bool {
        // '/' followed by an identifier character
        if self.pos + 1 < self.source.len() {
            is_ident_start(self.source[self.pos + 1])
        } else {
            false
        }
    }

    fn current(&self) -> char {
        self.source[self.pos]
    }

    fn advance(&mut self) {
        if self.pos < self.source.len() {
            self.byte_pos += self.source[self.pos].len_utf8();
            self.pos += 1;
        }
    }
}

fn is_ident_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_'
}

fn is_ident_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lex(input: &str) -> Vec<TokenKind> {
        Lexer::new(input)
            .tokenize()
            .into_iter()
            .map(|t| t.kind)
            .filter(|k| !matches!(k, TokenKind::Eof))
            .collect()
    }

    #[test]
    fn test_variable_declaration() {
        let tokens = lex("[V] — a 1D vector");
        assert_eq!(tokens[0], TokenKind::LBracket);
        assert_eq!(tokens[1], TokenKind::Identifier("V".into()));
        assert_eq!(tokens[2], TokenKind::RBracket);
        assert_eq!(tokens[3], TokenKind::EmDash);
    }

    #[test]
    fn test_step() {
        let tokens = lex("[1] — Do this thing");
        assert_eq!(tokens[0], TokenKind::LBracket);
        assert_eq!(tokens[1], TokenKind::Number(1));
        assert_eq!(tokens[2], TokenKind::RBracket);
        assert_eq!(tokens[3], TokenKind::EmDash);
    }

    #[test]
    fn test_special_markers() {
        let tokens = lex("[~] — [P] < [L]");
        assert_eq!(tokens[0], TokenKind::LBracket);
        assert_eq!(tokens[1], TokenKind::Tilde);
        assert_eq!(tokens[2], TokenKind::RBracket);
        assert_eq!(tokens[3], TokenKind::EmDash);
    }

    #[test]
    fn test_comment() {
        let tokens = lex("[^] this is a comment");
        assert_eq!(tokens[0], TokenKind::LBracket);
        assert_eq!(tokens[1], TokenKind::Caret);
        assert_eq!(tokens[2], TokenKind::RBracket);
    }

    #[test]
    fn test_function() {
        let tokens = lex("/add [A]: an integer");
        assert_eq!(tokens[0], TokenKind::Slash);
        assert_eq!(tokens[1], TokenKind::Identifier("add".into()));
        assert_eq!(tokens[2], TokenKind::LBracket);
        assert_eq!(tokens[3], TokenKind::Identifier("A".into()));
        assert_eq!(tokens[4], TokenKind::RBracket);
        assert_eq!(tokens[5], TokenKind::Colon);
    }

    #[test]
    fn test_conditional() {
        let tokens = lex("[1] — [A] > [B] ? DoX | DoY");
        assert!(tokens.contains(&TokenKind::Question));
        assert!(tokens.contains(&TokenKind::Pipe));
    }

    #[test]
    fn test_decorator() {
        let tokens = lex("[V] — a vector #mutable #type:list");
        assert!(tokens.contains(&TokenKind::Hash));
        assert!(tokens.contains(&TokenKind::Identifier("mutable".into())));
    }

    #[test]
    fn test_parallel() {
        let tokens = lex("[&]");
        assert_eq!(tokens[0], TokenKind::LBracket);
        assert_eq!(tokens[1], TokenKind::Ampersand);
        assert_eq!(tokens[2], TokenKind::RBracket);
    }

    #[test]
    fn test_inline_assignment() {
        let tokens = lex("[L]: length of [V] = 0");
        assert_eq!(tokens[0], TokenKind::LBracket);
        assert_eq!(tokens[1], TokenKind::Identifier("L".into()));
        assert_eq!(tokens[2], TokenKind::RBracket);
        assert_eq!(tokens[3], TokenKind::Colon);
        assert!(tokens.contains(&TokenKind::Equals));
        assert!(tokens.contains(&TokenKind::Number(0)));
    }

    #[test]
    fn test_loop_for_each() {
        let tokens = lex("[~] — [X] in [V]");
        assert_eq!(tokens[1], TokenKind::Tilde);
        assert!(tokens.contains(&TokenKind::In));
    }

    #[test]
    fn test_return() {
        let tokens = lex("[>] [R]");
        assert_eq!(tokens[1], TokenKind::GreaterThan);
    }

    #[test]
    fn test_log() {
        let tokens = lex("[!] — done processing");
        assert_eq!(tokens[1], TokenKind::Bang);
        assert_eq!(tokens[3], TokenKind::EmDash);
    }

    #[test]
    fn test_access_operator() {
        let tokens = lex("[V]@[P]");
        assert_eq!(tokens[0], TokenKind::LBracket);
        assert_eq!(tokens[1], TokenKind::Identifier("V".into()));
        assert_eq!(tokens[2], TokenKind::RBracket);
        assert_eq!(tokens[3], TokenKind::At);
        assert_eq!(tokens[4], TokenKind::LBracket);
        assert_eq!(tokens[5], TokenKind::Identifier("P".into()));
        assert_eq!(tokens[6], TokenKind::RBracket);
    }
}
