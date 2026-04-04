use jaw_parse::ast::*;
use jaw_parse::token::Span;
use serde_json::{json, Value};

/// Find the definition of a variable referenced at the given byte offset.
pub fn goto_definition(ast: &Source, source: &str, offset: usize) -> Option<Value> {
    // First, figure out what identifier is at the offset
    let ident = find_identifier_at(source, offset)?;

    // Search for the definition
    for item in &ast.items {
        match item {
            TopLevel::Variable(v) => {
                if v.name == ident {
                    return Some(location_response(&v.span, source));
                }
            }
            TopLevel::Function(f) => {
                if f.name == ident {
                    return Some(location_response(&f.span, source));
                }
                // Check function args
                for arg in &f.args {
                    if arg.name == ident {
                        return Some(location_response(&arg.span, source));
                    }
                }
                // Check inline assigns in function body
                if let Some(loc) = search_block_for_def(&f.body, &ident, source) {
                    return Some(loc);
                }
            }
            _ => {}
        }
    }

    None
}

fn search_block_for_def(block: &CodeBlock, ident: &str, source: &str) -> Option<Value> {
    for item in &block.items {
        match item {
            BlockItem::InlineAssign(a) if a.name == ident => {
                return Some(location_response(&a.span, source));
            }
            BlockItem::Loop(lp) => {
                if let Some(loc) = search_block_for_def(&lp.body, ident, source) {
                    return Some(loc);
                }
            }
            BlockItem::Parallel(par) => {
                if let Some(loc) = search_block_for_def(&par.body, ident, source) {
                    return Some(loc);
                }
            }
            _ => {}
        }
    }
    None
}

/// Extract the identifier name at a given offset in the source.
/// Looks for [IDENT] patterns.
fn find_identifier_at(source: &str, offset: usize) -> Option<String> {
    let bytes = source.as_bytes();

    // Walk backwards to find [
    let mut start = offset;
    while start > 0 && bytes[start - 1] != b'[' {
        if bytes[start - 1] == b']' || bytes[start - 1] == b'\n' {
            break;
        }
        start -= 1;
    }

    // Walk forward to find ]
    let mut end = offset;
    while end < bytes.len() && bytes[end] != b']' {
        if bytes[end] == b'[' || bytes[end] == b'\n' {
            break;
        }
        end += 1;
    }

    if start > 0 && bytes[start - 1] == b'[' && end < bytes.len() && bytes[end] == b']' {
        let ident = &source[start..end];
        if ident.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') && !ident.is_empty() {
            return Some(ident.to_string());
        }
    }

    None
}

fn offset_to_position(source: &str, offset: usize) -> (u32, u32) {
    let mut line = 0u32;
    let mut col = 0u32;
    for (i, ch) in source.char_indices() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }
    (line, col)
}

fn location_response(span: &Span, source: &str) -> Value {
    let (start_line, start_char) = offset_to_position(source, span.start);
    let (end_line, end_char) = offset_to_position(source, span.end);

    json!({
        "range": {
            "start": { "line": start_line, "character": start_char },
            "end": { "line": end_line, "character": end_char }
        }
    })
}
