use jaw_parse::ast::*;
use jaw_parse::token::Span;
use serde_json::{json, Value};

/// Find hover information at a given byte offset in the source.
pub fn hover_at(ast: &Source, source: &str, offset: usize) -> Option<Value> {
    // Search top-level items
    for item in &ast.items {
        match item {
            TopLevel::Variable(v) => {
                if contains_offset(&v.span, offset) {
                    let mut content = format!("**[{}]** — {}", v.name, v.description);
                    if !v.decorators.is_empty() {
                        content.push_str("\n\nDecorators: ");
                        for d in &v.decorators {
                            content.push_str(&format!("#{}", d.name));
                            if let Some(ref val) = d.value {
                                content.push_str(&format!(":{}", val));
                            }
                            content.push(' ');
                        }
                    }
                    return Some(hover_response(&content, &v.span, source));
                }
            }
            TopLevel::Function(f) => {
                if contains_offset(&f.span, offset) {
                    let args_str: Vec<String> = f
                        .args
                        .iter()
                        .map(|a| {
                            let mut s = format!("[{}]: {}", a.name, a.description);
                            if let Some(ref v) = a.value {
                                s.push_str(&format!(" = {}", v));
                            }
                            s
                        })
                        .collect();
                    let content = format!("**/{name}**\n\n{args}", name = f.name, args = args_str.join(", "));
                    return Some(hover_response(&content, &f.span, source));
                }
            }
            _ => {}
        }
    }

    None
}

fn contains_offset(span: &Span, offset: usize) -> bool {
    offset >= span.start && offset < span.end
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

fn hover_response(content: &str, span: &Span, source: &str) -> Value {
    let (start_line, start_char) = offset_to_position(source, span.start);
    let (end_line, end_char) = offset_to_position(source, span.end);

    json!({
        "contents": {
            "kind": "markdown",
            "value": content
        },
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
    fn function_hover_lists_all_multiline_args() {
        // Regression: hovering a function with one-arg-per-line declarations
        // (like `/Simulate` in the genetic-algorithm sample) used to show
        // only the first argument's description.
        let source = "/Simulate\n[G]: the population generation\n[E]: the environment\n[T]: simulation steps\n\t[>] [G]\n";
        let (ast, _) = parse(source);
        let offset = source.find("Simulate").unwrap();
        let result = hover_at(&ast, source, offset).expect("expected hover on /Simulate");
        let value = result["contents"]["value"].as_str().unwrap();
        assert!(value.contains("[G]: the population generation"), "missing G: {value}");
        assert!(value.contains("[E]: the environment"), "missing E: {value}");
        assert!(value.contains("[T]: simulation steps"), "missing T: {value}");
    }
}
