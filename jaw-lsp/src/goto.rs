use jaw_parse::ast::*;
use jaw_parse::token::Span;
use serde_json::{json, Value};

/// Find the definition of a variable referenced at the given byte offset.
///
/// Scoping rules: prefer the enclosing function's args and inline-assigns
/// over top-level variables. Never resolve a reference to a same-named
/// variable inside a *different* function.
pub fn goto_definition(ast: &Source, source: &str, offset: usize) -> Option<Value> {
    let ident = find_identifier_at(source, offset)?;

    if let Some(func) = find_enclosing_function(ast, offset) {
        if let Some(loc) = find_in_function(func, &ident, source) {
            return Some(loc);
        }
    }

    for item in &ast.items {
        if let TopLevel::Variable(v) = item {
            if v.name == ident {
                return Some(location_response(&v.span, source));
            }
        }
    }

    None
}

fn find_enclosing_function(ast: &Source, offset: usize) -> Option<&Function> {
    for item in &ast.items {
        if let TopLevel::Function(f) = item {
            if offset >= f.span.start && offset < f.span.end {
                return Some(f);
            }
        }
    }
    None
}

fn find_in_function(f: &Function, ident: &str, source: &str) -> Option<Value> {
    for arg in &f.args {
        if arg.name == ident {
            return Some(location_response(&arg.span, source));
        }
    }
    search_block_for_def(&f.body, ident, source)
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

#[cfg(test)]
mod tests {
    use super::*;
    use jaw_parse::parse;

    #[test]
    fn resolves_arg_within_same_function() {
        let source = "/classify [X]: a number\n\t[>] [X]\n\n/handle [X]: a number\n\t[>] [X] * 2\n";
        let (ast, _diags) = parse(source);

        // `[X]` on the `[>]` line of /handle — second function.
        let offset = source.find("[>] [X] * 2").unwrap() + 5;
        let loc = goto_definition(&ast, source, offset).expect("definition found");

        // It must point at /handle's [X], which is on line 3 (0-indexed), not /classify's line 0.
        let line = loc["range"]["start"]["line"].as_u64().unwrap();
        assert_eq!(line, 3, "expected /handle's [X] on line 3, got line {}", line);
    }

    #[test]
    fn falls_back_to_top_level_variable() {
        let source = "[V] — a vector\n[T] — a threshold\n\n/use [X]: a number\n\t[>] [V] + [X]\n";
        let (ast, _diags) = parse(source);

        // Click on `[V]` inside /use.
        let offset = source.find("[>] [V]").unwrap() + 5;
        let loc = goto_definition(&ast, source, offset).expect("definition found");

        let line = loc["range"]["start"]["line"].as_u64().unwrap();
        assert_eq!(line, 0, "expected top-level [V] on line 0, got line {}", line);
    }

    #[test]
    fn does_not_leak_across_functions() {
        // /a has [X] but /b does not — clicking [X] in /b must NOT resolve to /a.
        let source = "/a [X]: a number\n\t[>] [X]\n\n/b\n[Y]: a thing\n\t[>] [X]\n";
        let (ast, _diags) = parse(source);

        let offset = source.rfind("[X]").unwrap() + 1;
        let loc = goto_definition(&ast, source, offset);

        assert!(
            loc.is_none(),
            "expected no definition (dangling ref), got {:?}",
            loc
        );
    }

    #[test]
    fn resolves_inline_assign_inside_function_body() {
        // An inline-assign declared as its own line in the function body, then
        // referenced later in a step expression, should resolve back to the assign.
        let source = "/process\n[V]: a list\n\t[R]: results\n\t[1] — [R]\n";
        let (ast, _diags) = parse(source);

        // Point at the `R` inside `[R]` on the step line.
        let ref_offset = source.rfind("[R]").unwrap() + 1;
        let loc = goto_definition(&ast, source, ref_offset).expect("definition found");
        let line = loc["range"]["start"]["line"].as_u64().unwrap();
        // [R]: results is on line 2 (0-indexed).
        assert_eq!(line, 2);
    }

}
