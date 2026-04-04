pub mod ast;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod token;

use ast::Source;
use error::Diagnostic;
use lexer::Lexer;
use parser::Parser;

/// Parse JAW source code into an AST with diagnostics.
pub fn parse(source: &str) -> (Source, Vec<Diagnostic>) {
    let tokens = Lexer::new(source).tokenize();
    Parser::new(source, tokens).parse()
}
