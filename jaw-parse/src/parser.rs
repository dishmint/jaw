use crate::ast::*;
use crate::error::Diagnostic;
use crate::token::{Span, Token, TokenKind};

pub struct Parser {
    source: String,
    tokens: Vec<Token>,
    pos: usize,
    diagnostics: Vec<Diagnostic>,
}

impl Parser {
    pub fn new(source: &str, tokens: Vec<Token>) -> Self {
        Self {
            source: source.to_string(),
            tokens,
            pos: 0,
            diagnostics: Vec::new(),
        }
    }

    pub fn parse(mut self) -> (Source, Vec<Diagnostic>) {
        let items = self.parse_source();
        (Source { items }, self.diagnostics)
    }

    fn parse_source(&mut self) -> Vec<TopLevel> {
        let mut items = Vec::new();

        self.skip_newlines();

        while !self.at_eof() {
            if let Some(item) = self.parse_top_level() {
                items.push(item);
            }
            self.skip_newlines();
        }

        items
    }

    fn parse_top_level(&mut self) -> Option<TopLevel> {
        // Look at the current token to decide what to parse
        match self.peek_kind() {
            // Function definition: /name
            Some(TokenKind::Slash) => self.parse_function().map(TopLevel::Function),

            // Something starting with [
            Some(TokenKind::LBracket) => {
                // Peek inside the bracket to determine what this is
                match self.peek_bracket_content() {
                    BracketContent::Identifier => {
                        // Could be variable declaration or inline assignment
                        // Check what follows the ]
                        if self.is_inline_assignment() {
                            // Inline assignment at top level — treat as text/variable
                            self.parse_variable_decl().map(TopLevel::Variable)
                        } else {
                            self.parse_variable_decl().map(TopLevel::Variable)
                        }
                    }
                    BracketContent::Number => {
                        // Step at top level — unusual but valid.
                        // Reuse the in-block step parser with block_indent = 0.
                        match self.parse_step_or_complex_cond(0) {
                            Some(BlockItem::Step(s)) => Some(TopLevel::Step(s)),
                            _ => None,
                        }
                    }
                    BracketContent::Caret => self.parse_comment().map(TopLevel::Comment),
                    BracketContent::Asterisk => self.parse_comment().map(TopLevel::Comment),
                    _ => {
                        self.skip_to_newline();
                        None
                    }
                }
            }

            Some(TokenKind::Indent(_)) => {
                self.advance();
                self.parse_top_level()
            }

            _ => {
                // Consume as text
                let start = self.current_span();
                let mut text = String::new();
                while !self.at_eof() && !self.at_newline() {
                    text.push_str(&self.token_text());
                    self.advance();
                }
                if text.trim().is_empty() {
                    None
                } else {
                    Some(TopLevel::Text(TextNode {
                        text,
                        span: start.merge(self.prev_span()),
                    }))
                }
            }
        }
    }

    fn parse_variable_decl(&mut self) -> Option<Variable> {
        let start = self.current_span();

        // Expect [
        if !self.expect(TokenKind::LBracket) {
            return None;
        }

        // Expect identifier
        let name = match self.peek_kind() {
            Some(TokenKind::Identifier(ref s)) => {
                let s = s.clone();
                self.advance();
                s
            }
            _ => {
                self.error("expected identifier", self.current_span());
                self.skip_to_newline();
                return None;
            }
        };

        // Expect ]
        if !self.expect(TokenKind::RBracket) {
            self.skip_to_newline();
            return None;
        }

        // Check for : (inline assignment) vs — (variable declaration)
        match self.peek_kind() {
            Some(TokenKind::Colon) => {
                // This is actually an inline assignment at top level
                self.advance(); // consume :
                let desc = self.consume_text_until_equals_or_newline();
                let _value = if matches!(self.peek_kind(), Some(TokenKind::Equals)) {
                    self.advance();
                    Some(self.consume_rest_of_line())
                } else {
                    None
                };
                // Wrap as variable for top-level
                Some(Variable {
                    name,
                    description: desc,
                    decorators: Vec::new(),
                    span: start.merge(self.prev_span()),
                })
            }
            Some(TokenKind::EmDash) => {
                self.advance(); // consume —
                let desc = self.consume_text_until_decorator_or_newline();
                let decorators = self.parse_decorators();
                Some(Variable {
                    name,
                    description: desc,
                    decorators,
                    span: start.merge(self.prev_span()),
                })
            }
            _ => {
                let desc = self.consume_rest_of_line();
                Some(Variable {
                    name,
                    description: desc,
                    decorators: Vec::new(),
                    span: start.merge(self.prev_span()),
                })
            }
        }
    }

    fn parse_function(&mut self) -> Option<Function> {
        let start = self.current_span();

        // Expect /
        if !self.expect(TokenKind::Slash) {
            return None;
        }

        // Expect function name
        let name = match self.peek_kind() {
            Some(TokenKind::Identifier(ref s)) => {
                let s = s.clone();
                self.advance();
                s
            }
            _ => {
                self.error("expected function name", self.current_span());
                self.skip_to_newline();
                return None;
            }
        };

        // Optional decorators on function line
        let decorators = self.parse_decorators();

        // Parse arguments (may be on same line or next line)
        let args = self.parse_function_args();

        // Skip to body
        self.skip_newlines();

        // Parse body (indented block)
        let body = self.parse_code_block();

        Some(Function {
            name,
            decorators,
            args,
            body,
            span: start.merge(self.prev_span()),
        })
    }

    fn parse_function_args(&mut self) -> Vec<InlineAssign> {
        let mut args = Vec::new();

        // Args can be on the same line as /name or on the next line
        // They look like: [A]: description, [B]: description = value
        while matches!(self.peek_kind(), Some(TokenKind::LBracket)) {
            if let Some(arg) = self.parse_inline_assign_arg() {
                args.push(arg);
            }
            // Consume comma separator
            if matches!(self.peek_kind(), Some(TokenKind::Comma)) {
                self.advance();
            } else {
                break;
            }
        }

        // Check next line for args if none found and we're at a newline
        if args.is_empty() && self.at_newline() {
            let saved = self.pos;
            self.skip_newlines();

            // Check if next line starts with indent + [
            if matches!(self.peek_kind(), Some(TokenKind::Indent(_))) {
                self.advance();
            }

            if matches!(self.peek_kind(), Some(TokenKind::LBracket)) && self.is_inline_assignment() {
                while matches!(self.peek_kind(), Some(TokenKind::LBracket)) {
                    if let Some(arg) = self.parse_inline_assign_arg() {
                        args.push(arg);
                    }
                    if matches!(self.peek_kind(), Some(TokenKind::Comma)) {
                        self.advance();
                    } else {
                        break;
                    }
                }
            } else {
                // Not args, restore position
                self.pos = saved;
            }
        }

        args
    }

    fn parse_inline_assign_arg(&mut self) -> Option<InlineAssign> {
        let start = self.current_span();

        if !self.expect(TokenKind::LBracket) {
            return None;
        }

        let name = match self.peek_kind() {
            Some(TokenKind::Identifier(ref s)) => {
                let s = s.clone();
                self.advance();
                s
            }
            _ => return None,
        };

        if !self.expect(TokenKind::RBracket) {
            return None;
        }

        if !self.expect_kind(&TokenKind::Colon) {
            return None;
        }

        let desc = self.consume_text_until(&[TokenKind::Equals, TokenKind::Comma]);
        let value = if matches!(self.peek_kind(), Some(TokenKind::Equals)) {
            self.advance();
            Some(self.consume_value_with_balanced_brackets())
        } else {
            None
        };

        Some(InlineAssign {
            name,
            description: desc,
            value,
            span: start.merge(self.prev_span()),
        })
    }

    fn parse_code_block(&mut self) -> CodeBlock {
        let mut items = Vec::new();
        let block_indent = self.current_indent_level();

        loop {
            self.skip_newlines();

            if self.at_eof() {
                break;
            }

            // Check indent level
            let indent = self.current_indent_level();
            if indent < block_indent && block_indent > 0 {
                break;
            }

            // Consume indent token if present
            if matches!(self.peek_kind(), Some(TokenKind::Indent(_))) {
                self.advance();
            }

            if self.at_eof() || self.at_newline() {
                break;
            }

            match self.peek_kind() {
                Some(TokenKind::LBracket) => {
                    match self.peek_bracket_content() {
                        BracketContent::Number => {
                            // Could be step or complex conditional
                            if let Some(item) = self.parse_step_or_complex_cond(block_indent) {
                                items.push(item);
                            }
                        }
                        BracketContent::Identifier => {
                            // Inline assignment inside block
                            if let Some(assign) = self.parse_block_inline_assign() {
                                items.push(BlockItem::InlineAssign(assign));
                            }
                        }
                        BracketContent::Tilde => {
                            if let Some(lp) = self.parse_loop(block_indent) {
                                items.push(BlockItem::Loop(lp));
                            }
                        }
                        BracketContent::Ampersand => {
                            if let Some(par) = self.parse_parallel(block_indent) {
                                items.push(BlockItem::Parallel(par));
                            }
                        }
                        BracketContent::Caret | BracketContent::Asterisk => {
                            if let Some(c) = self.parse_comment() {
                                items.push(BlockItem::Comment(c));
                            }
                        }
                        BracketContent::Bang => {
                            if let Some(log) = self.parse_log() {
                                items.push(BlockItem::Log(log));
                            }
                        }
                        BracketContent::GreaterThan => {
                            if let Some(ret) = self.parse_return() {
                                items.push(BlockItem::Return(ret));
                            }
                        }
                        BracketContent::Plus | BracketContent::Minus => {
                            // Stray [+]/[-] outside a complex cond — skip
                            self.skip_to_newline();
                        }
                        BracketContent::Other => {
                            self.skip_to_newline();
                        }
                    }
                }
                Some(TokenKind::Slash) => {
                    // Nested function or function call — break out of block
                    break;
                }
                _ => {
                    self.skip_to_newline();
                }
            }
        }

        CodeBlock { items }
    }

    fn parse_step_or_complex_cond(&mut self, block_indent: usize) -> Option<BlockItem> {
        let start = self.current_span();

        // Parse [N] —
        self.advance(); // [
        let number = match self.peek_kind() {
            Some(TokenKind::Number(n)) => {
                let n = n;
                self.advance();
                n
            }
            _ => {
                self.skip_to_newline();
                return None;
            }
        };
        self.advance(); // ]

        if !self.expect(TokenKind::EmDash) {
            self.skip_to_newline();
            return None;
        }

        // Collect the rest of the line as expression text
        let expr_text = self.consume_text_until_decorator_or_newline();
        let decorators = self.parse_decorators();

        // Check if line ends with ? (potential complex conditional)
        let trimmed = expr_text.trim();
        if trimmed.ends_with('?') {
            // Peek at next lines for [+]/[-]
            let saved = self.pos;
            self.skip_newlines();

            let next_indent = self.current_indent_level();
            if next_indent > block_indent {
                if matches!(self.peek_bracket_content_after_indent(), Some(BracketContent::Plus)) {
                    // Complex conditional
                    let condition = trimmed.trim_end_matches('?').trim().to_string();
                    let true_branch = self.parse_branch_block();
                    let false_branch = self.parse_branch_block();

                    return Some(BlockItem::ComplexCond(ComplexCond {
                        step_number: number,
                        condition,
                        true_branch,
                        false_branch,
                        span: start.merge(self.prev_span()),
                    }));
                }
            }
            self.pos = saved;
        }

        // Check if expression contains ? for inline conditional
        let expression = if trimmed.contains('?') {
            self.parse_conditional_expr(trimmed)
        } else {
            Expression::Code(expr_text.trim().to_string())
        };

        Some(BlockItem::Step(Step {
            number,
            expression,
            decorators,
            span: start.merge(self.prev_span()),
        }))
    }

    fn parse_conditional_expr(&self, text: &str) -> Expression {
        // Parse: CODE ? CODE ( | CODE ? CODE )* ( | CODE )?
        let mut branches = Vec::new();
        let mut remaining = text;

        // Split on | first, then handle ? within each segment
        let segments: Vec<&str> = remaining.splitn(2, '?').collect();
        if segments.len() < 2 {
            return Expression::Code(text.to_string());
        }

        let first_condition = segments[0].trim().to_string();
        remaining = segments[1].trim();

        // Now split the rest by |
        let parts: Vec<&str> = remaining.split('|').collect();

        if parts.is_empty() {
            return Expression::Code(text.to_string());
        }

        // First part is the consequence of the first condition
        branches.push(CondBranch {
            condition: first_condition,
            consequence: parts[0].trim().to_string(),
        });

        // Remaining parts: either "cond ? consequence" or just "else_value"
        let mut else_branch = None;
        for part in &parts[1..] {
            let part = part.trim();
            if let Some(q_pos) = part.find('?') {
                let cond = part[..q_pos].trim().to_string();
                let cons = part[q_pos + 1..].trim().to_string();
                branches.push(CondBranch {
                    condition: cond,
                    consequence: cons,
                });
            } else {
                else_branch = Some(part.to_string());
            }
        }

        let span = Span::new(0, 0); // Placeholder — real span would need tracking
        Expression::Conditional(Conditional {
            branches,
            else_branch,
            span,
        })
    }

    fn parse_branch_block(&mut self) -> String {
        self.skip_newlines();
        if matches!(self.peek_kind(), Some(TokenKind::Indent(_))) {
            self.advance();
        }

        // Expect [+] or [-]
        if matches!(self.peek_kind(), Some(TokenKind::LBracket)) {
            self.advance(); // [
            self.advance(); // + or -
            self.advance(); // ]
        }

        if matches!(self.peek_kind(), Some(TokenKind::EmDash)) {
            self.advance();
        }

        self.consume_rest_of_line()
    }

    fn parse_block_inline_assign(&mut self) -> Option<InlineAssign> {
        let start = self.current_span();

        self.advance(); // [
        let name = match self.peek_kind() {
            Some(TokenKind::Identifier(ref s)) => {
                let s = s.clone();
                self.advance();
                s
            }
            _ => {
                self.skip_to_newline();
                return None;
            }
        };
        self.advance(); // ]

        if !self.expect_kind(&TokenKind::Colon) {
            self.skip_to_newline();
            return None;
        }

        let desc = self.consume_text_until_equals_or_newline();
        let value = if matches!(self.peek_kind(), Some(TokenKind::Equals)) {
            self.advance();
            Some(self.consume_rest_of_line())
        } else {
            None
        };

        Some(InlineAssign {
            name,
            description: desc,
            value,
            span: start.merge(self.prev_span()),
        })
    }

    fn parse_loop(&mut self, _parent_indent: usize) -> Option<Loop> {
        let start = self.current_span();

        // Consume [~] —
        self.advance(); // [
        self.advance(); // ~
        self.advance(); // ]

        if !self.expect(TokenKind::EmDash) {
            self.skip_to_newline();
            return None;
        }

        // Parse loop expression
        let expr_text = self.consume_rest_of_line();
        let expr = self.parse_loop_expr(&expr_text);

        self.skip_newlines();
        let body = self.parse_code_block();

        Some(Loop {
            expr,
            body,
            span: start.merge(self.prev_span()),
        })
    }

    fn parse_loop_expr(&self, text: &str) -> LoopExpr {
        let trimmed = text.trim();

        // Check for "X in Y" or "(A, B) in Y" pattern
        if let Some(in_pos) = trimmed.find(" in ") {
            let lhs = trimmed[..in_pos].trim();
            let rhs = trimmed[in_pos + 4..].trim();

            // Check for destructured: (A, B, ...)
            let vars = if lhs.starts_with('(') && lhs.ends_with(')') {
                let inner = &lhs[1..lhs.len() - 1];
                inner
                    .split(',')
                    .map(|s| {
                        let s = s.trim();
                        // Strip brackets if present: [A] -> A
                        if s.starts_with('[') && s.ends_with(']') {
                            s[1..s.len() - 1].to_string()
                        } else {
                            s.to_string()
                        }
                    })
                    .collect()
            } else {
                // Single variable
                let var = if lhs.starts_with('[') && lhs.ends_with(']') {
                    lhs[1..lhs.len() - 1].to_string()
                } else {
                    lhs.to_string()
                };
                vec![var]
            };

            let iterable = if rhs.starts_with('[') && rhs.ends_with(']') {
                rhs[1..rhs.len() - 1].to_string()
            } else {
                rhs.to_string()
            };

            LoopExpr::ForEach { vars, iterable }
        } else {
            LoopExpr::While(trimmed.to_string())
        }
    }

    fn parse_parallel(&mut self, _parent_indent: usize) -> Option<Parallel> {
        let start = self.current_span();

        // Consume [&]
        self.advance(); // [
        self.advance(); // &
        self.advance(); // ]

        self.skip_newlines();
        let body = self.parse_code_block();

        Some(Parallel {
            body,
            span: start.merge(self.prev_span()),
        })
    }

    fn parse_comment(&mut self) -> Option<Comment> {
        let start = self.current_span();

        // Consume [ ^ or * ]
        self.advance(); // [
        let is_code = matches!(self.peek_kind(), Some(TokenKind::Caret));
        self.advance(); // ^ or *
        self.advance(); // ]

        let mut text = self.consume_rest_of_line();

        // Multi-line continuation: consume following lines that don't start with a JAW marker
        loop {
            let saved = self.pos;
            self.skip_newlines();

            if self.at_eof() {
                break;
            }

            // Check if next line starts with a JAW marker
            let next = self.peek_kind_skip_indent();
            match next {
                Some(TokenKind::LBracket) | Some(TokenKind::Slash) | None => {
                    // It's a JAW construct or EOF — stop continuation
                    self.pos = saved;
                    break;
                }
                Some(TokenKind::Indent(_)) => {
                    self.pos = saved;
                    break;
                }
                _ => {
                    // Plain text continuation line
                    let line = self.consume_rest_of_line();
                    text.push('\n');
                    text.push_str(&line);
                }
            }
        }

        let span = start.merge(self.prev_span());
        if is_code {
            Some(Comment::Code { text: text.trim().to_string(), span })
        } else {
            Some(Comment::General { text: text.trim().to_string(), span })
        }
    }

    fn parse_log(&mut self) -> Option<Log> {
        let start = self.current_span();

        // Consume [!] —
        self.advance(); // [
        self.advance(); // !
        self.advance(); // ]

        if matches!(self.peek_kind(), Some(TokenKind::EmDash)) {
            self.advance();
        }

        let text = self.consume_rest_of_line();

        Some(Log {
            text: text.trim().to_string(),
            span: start.merge(self.prev_span()),
        })
    }

    fn parse_return(&mut self) -> Option<Return> {
        let start = self.current_span();

        // Consume [>]
        self.advance(); // [
        self.advance(); // >
        self.advance(); // ]

        let value = self.consume_text_until_decorator_or_newline();
        let decorators = self.parse_decorators();

        Some(Return {
            value: value.trim().to_string(),
            decorators,
            span: start.merge(self.prev_span()),
        })
    }

    fn parse_decorators(&mut self) -> Vec<Decorator> {
        let mut decorators = Vec::new();

        while matches!(self.peek_kind(), Some(TokenKind::Hash)) {
            let start = self.current_span();
            self.advance(); // #

            let name = match self.peek_kind() {
                Some(TokenKind::Identifier(ref s)) => {
                    let s = s.clone();
                    self.advance();
                    s
                }
                _ => break,
            };

            let value = if matches!(self.peek_kind(), Some(TokenKind::Colon)) {
                self.advance();
                match self.peek_kind() {
                    Some(TokenKind::Identifier(ref s)) => {
                        let s = s.clone();
                        self.advance();
                        Some(s)
                    }
                    Some(TokenKind::Text(ref s)) => {
                        let s = s.clone();
                        self.advance();
                        Some(s)
                    }
                    _ => None,
                }
            } else {
                None
            };

            decorators.push(Decorator {
                name,
                value,
                span: start.merge(self.prev_span()),
            });
        }

        decorators
    }

    // -- Helper methods --

    fn peek_kind(&self) -> Option<TokenKind> {
        self.tokens.get(self.pos).map(|t| t.kind.clone())
    }

    fn peek_kind_skip_indent(&self) -> Option<TokenKind> {
        let mut i = self.pos;
        while i < self.tokens.len() {
            match &self.tokens[i].kind {
                TokenKind::Indent(_) | TokenKind::Newline => i += 1,
                other => return Some(other.clone()),
            }
        }
        None
    }

    fn peek_bracket_content(&self) -> BracketContent {
        // Look at [pos] = LBracket, [pos+1] = content
        if self.pos + 1 >= self.tokens.len() {
            return BracketContent::Other;
        }
        match &self.tokens[self.pos + 1].kind {
            TokenKind::Identifier(_) => BracketContent::Identifier,
            TokenKind::Number(_) => BracketContent::Number,
            TokenKind::Tilde => BracketContent::Tilde,
            TokenKind::Ampersand => BracketContent::Ampersand,
            TokenKind::Caret => BracketContent::Caret,
            TokenKind::Asterisk => BracketContent::Asterisk,
            TokenKind::Bang => BracketContent::Bang,
            TokenKind::GreaterThan => BracketContent::GreaterThan,
            TokenKind::Plus => BracketContent::Plus,
            TokenKind::Minus => BracketContent::Minus,
            _ => BracketContent::Other,
        }
    }

    fn peek_bracket_content_after_indent(&self) -> Option<BracketContent> {
        let mut i = self.pos;
        while i < self.tokens.len() {
            match &self.tokens[i].kind {
                TokenKind::Indent(_) | TokenKind::Newline => i += 1,
                TokenKind::LBracket => {
                    if i + 1 < self.tokens.len() {
                        return Some(match &self.tokens[i + 1].kind {
                            TokenKind::Plus => BracketContent::Plus,
                            TokenKind::Minus => BracketContent::Minus,
                            _ => BracketContent::Other,
                        });
                    }
                    return None;
                }
                _ => return None,
            }
        }
        None
    }

    fn is_inline_assignment(&self) -> bool {
        // Check if after [ID] there's a :
        let mut i = self.pos;
        // Skip [ ID ]
        if i < self.tokens.len() && matches!(self.tokens[i].kind, TokenKind::LBracket) {
            i += 1;
        }
        if i < self.tokens.len() && matches!(self.tokens[i].kind, TokenKind::Identifier(_)) {
            i += 1;
        }
        if i < self.tokens.len() && matches!(self.tokens[i].kind, TokenKind::RBracket) {
            i += 1;
        }
        i < self.tokens.len() && matches!(self.tokens[i].kind, TokenKind::Colon)
    }

    fn current_indent_level(&self) -> usize {
        // Look backwards for the most recent indent token on this line
        if self.pos > 0 {
            let mut i = self.pos;
            // If current token is Indent, use it
            if let Some(TokenKind::Indent(n)) = self.peek_kind() {
                return n;
            }
            // Look back to find indent at start of current line
            while i > 0 {
                i -= 1;
                match &self.tokens[i].kind {
                    TokenKind::Indent(n) => return *n,
                    TokenKind::Newline => return 0,
                    _ => continue,
                }
            }
        }
        0
    }

    fn current_span(&self) -> Span {
        self.tokens
            .get(self.pos)
            .map(|t| t.span)
            .unwrap_or(Span::new(0, 0))
    }

    fn prev_span(&self) -> Span {
        if self.pos > 0 {
            self.tokens[self.pos - 1].span
        } else {
            Span::new(0, 0)
        }
    }

    fn advance(&mut self) {
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
    }

    fn at_eof(&self) -> bool {
        self.pos >= self.tokens.len()
            || matches!(self.peek_kind(), Some(TokenKind::Eof))
    }

    fn at_newline(&self) -> bool {
        matches!(self.peek_kind(), Some(TokenKind::Newline))
    }

    fn expect(&mut self, kind: TokenKind) -> bool {
        if self.peek_kind() == Some(kind.clone()) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect_kind(&mut self, kind: &TokenKind) -> bool {
        if let Some(ref k) = self.peek_kind() {
            if std::mem::discriminant(k) == std::mem::discriminant(kind) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn skip_newlines(&mut self) {
        while matches!(self.peek_kind(), Some(TokenKind::Newline)) {
            self.advance();
        }
    }

    fn skip_to_newline(&mut self) {
        while !self.at_eof() && !self.at_newline() {
            self.advance();
        }
    }

    fn token_text(&self) -> String {
        match self.peek_kind() {
            Some(TokenKind::Identifier(s)) => s,
            Some(TokenKind::Text(s)) => s,
            Some(TokenKind::Number(n)) => n.to_string(),
            Some(TokenKind::EmDash) => "—".to_string(),
            Some(TokenKind::Colon) => ":".to_string(),
            Some(TokenKind::Equals) => "=".to_string(),
            Some(TokenKind::Comma) => ",".to_string(),
            Some(TokenKind::Hash) => "#".to_string(),
            Some(TokenKind::At) => "@".to_string(),
            Some(TokenKind::Question) => "?".to_string(),
            Some(TokenKind::Pipe) => "|".to_string(),
            Some(TokenKind::Slash) => "/".to_string(),
            Some(TokenKind::LBracket) => "[".to_string(),
            Some(TokenKind::RBracket) => "]".to_string(),
            Some(TokenKind::LParen) => "(".to_string(),
            Some(TokenKind::RParen) => ")".to_string(),
            _ => String::new(),
        }
    }

    fn source_text(&self, start: usize, end: usize) -> String {
        self.source[start..end].to_string()
    }

    fn consume_rest_of_line(&mut self) -> String {
        let start = self.current_span().start;
        while !self.at_eof() && !self.at_newline() {
            self.advance();
        }
        let end = self.prev_span().end;
        if end > start {
            self.source_text(start, end)
        } else {
            String::new()
        }
    }

    fn consume_text_until_decorator_or_newline(&mut self) -> String {
        let start = self.current_span().start;
        while !self.at_eof() && !self.at_newline() && !matches!(self.peek_kind(), Some(TokenKind::Hash)) {
            self.advance();
        }
        let end = self.prev_span().end;
        if end > start {
            self.source_text(start, end).trim().to_string()
        } else {
            String::new()
        }
    }

    fn consume_text_until_equals_or_newline(&mut self) -> String {
        let start = self.current_span().start;
        while !self.at_eof() && !self.at_newline() && !matches!(self.peek_kind(), Some(TokenKind::Equals)) {
            self.advance();
        }
        let end = self.prev_span().end;
        if end > start {
            self.source_text(start, end).trim().to_string()
        } else {
            String::new()
        }
    }

    /// Consume tokens until end-of-line or a top-level comma (depth 0).
    /// Bracket pairs `[` / `]` are tracked so commas inside them are
    /// preserved as part of the value (e.g. `Length[ [V], [W] ]`).
    fn consume_value_with_balanced_brackets(&mut self) -> String {
        let start = self.current_span().start;
        let mut depth: i32 = 0;
        while !self.at_eof() && !self.at_newline() {
            match self.peek_kind() {
                Some(TokenKind::LBracket) => depth += 1,
                Some(TokenKind::RBracket) => {
                    if depth > 0 {
                        depth -= 1;
                    }
                }
                Some(TokenKind::Comma) if depth == 0 => break,
                _ => {}
            }
            self.advance();
        }
        let end = self.prev_span().end;
        if end > start {
            self.source_text(start, end).trim().to_string()
        } else {
            String::new()
        }
    }

    fn consume_text_until(&mut self, stops: &[TokenKind]) -> String {
        let start = self.current_span().start;
        while !self.at_eof() && !self.at_newline() {
            if let Some(ref kind) = self.peek_kind() {
                if stops.iter().any(|s| std::mem::discriminant(s) == std::mem::discriminant(kind)) {
                    break;
                }
            }
            self.advance();
        }
        let end = self.prev_span().end;
        if end > start {
            self.source_text(start, end).trim().to_string()
        } else {
            String::new()
        }
    }

    fn error(&mut self, message: &str, span: Span) {
        self.diagnostics.push(Diagnostic::error(message, span));
    }
}

#[derive(Debug, Clone)]
enum BracketContent {
    Identifier,
    Number,
    Tilde,
    Ampersand,
    Caret,
    Asterisk,
    Bang,
    GreaterThan,
    Plus,
    Minus,
    Other,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse(input: &str) -> (Source, Vec<Diagnostic>) {
        let tokens = Lexer::new(input).tokenize();
        Parser::new(input, tokens).parse()
    }

    #[test]
    fn test_parse_variable() {
        let (ast, diags) = parse("[V] — a 1D vector");
        assert!(diags.is_empty());
        assert_eq!(ast.items.len(), 1);
        match &ast.items[0] {
            TopLevel::Variable(v) => {
                assert_eq!(v.name, "V");
                assert!(v.description.contains("1D vector"));
            }
            _ => panic!("expected variable"),
        }
    }

    #[test]
    fn test_parse_variable_with_decorator() {
        let (ast, diags) = parse("[V] — a vector #mutable #type:list");
        assert!(diags.is_empty());
        match &ast.items[0] {
            TopLevel::Variable(v) => {
                assert_eq!(v.decorators.len(), 2);
                assert_eq!(v.decorators[0].name, "mutable");
                assert_eq!(v.decorators[1].name, "type");
                assert_eq!(v.decorators[1].value, Some("list".to_string()));
            }
            _ => panic!("expected variable"),
        }
    }

    #[test]
    fn test_parse_function() {
        let (ast, diags) = parse("/add [A]: an integer, [B]: an integer\n\t[>] [A] + [B]");
        assert!(diags.is_empty());
        match &ast.items[0] {
            TopLevel::Function(f) => {
                assert_eq!(f.name, "add");
                assert_eq!(f.args.len(), 2);
                assert_eq!(f.args[0].name, "A");
                assert_eq!(f.args[1].name, "B");
            }
            _ => panic!("expected function"),
        }
    }

    #[test]
    fn test_parse_function_arg_value_with_balanced_brackets() {
        let (ast, diags) =
            parse("/foo [X]: count = Length[ [V], [W] ], [Y]: name\n\t[>] [X]");
        assert!(diags.is_empty(), "unexpected diags: {:?}", diags);
        match &ast.items[0] {
            TopLevel::Function(f) => {
                assert_eq!(f.args.len(), 2);
                assert_eq!(f.args[0].name, "X");
                assert_eq!(
                    f.args[0].value.as_deref(),
                    Some("Length[ [V], [W] ]"),
                    "value should include the inner comma"
                );
                assert_eq!(f.args[1].name, "Y");
                assert_eq!(f.args[1].description, "name");
            }
            _ => panic!("expected function"),
        }
    }

    #[test]
    fn test_parse_comment() {
        let (ast, _) = parse("[^] this is a code comment");
        match &ast.items[0] {
            TopLevel::Comment(Comment::Code { text, .. }) => {
                assert!(text.contains("code comment"));
            }
            _ => panic!("expected code comment"),
        }
    }

    #[test]
    fn test_parse_general_comment() {
        let (ast, _) = parse("[*] this is a general comment");
        match &ast.items[0] {
            TopLevel::Comment(Comment::General { text, .. }) => {
                assert!(text.contains("general comment"));
            }
            _ => panic!("expected general comment"),
        }
    }
}
