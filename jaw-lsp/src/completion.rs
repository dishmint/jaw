use std::collections::HashSet;

use jaw_parse::ast::*;
use serde_json::{json, Value};

/// LSP CompletionItemKind::Variable
const KIND_VARIABLE: u32 = 6;

/// Produce completion items at the given byte offset.
///
/// Only fires inside a `[ ... ]` bracket context — JAW variable references
/// always live between brackets. Enclosing-function locals (args +
/// inline-assigns) rank first; top-level variables fall back.
pub fn completion_at(ast: &Source, source: &str, offset: usize) -> Value {
    if !in_bracket_context(source, offset) {
        return empty_list();
    }

    let mut seen: HashSet<String> = HashSet::new();
    let mut items: Vec<Value> = Vec::new();

    if let Some(func) = find_enclosing_function(ast, offset) {
        for arg in &func.args {
            if seen.insert(arg.name.clone()) {
                items.push(item(
                    &arg.name,
                    &format!("arg of /{}", func.name),
                    &arg.description,
                ));
            }
        }
        collect_assigns(&func.body, &func.name, &mut seen, &mut items);
    }

    for top in &ast.items {
        if let TopLevel::Variable(v) = top {
            if seen.insert(v.name.clone()) {
                items.push(item(&v.name, "global", &v.description));
            }
        }
    }

    json!({ "isIncomplete": false, "items": items })
}

fn item(name: &str, detail: &str, doc: &str) -> Value {
    json!({
        "label": name,
        "kind": KIND_VARIABLE,
        "detail": detail,
        "documentation": doc,
        "insertText": name,
    })
}

fn collect_assigns(
    block: &CodeBlock,
    fn_name: &str,
    seen: &mut HashSet<String>,
    items: &mut Vec<Value>,
) {
    for b in &block.items {
        match b {
            BlockItem::InlineAssign(a) => {
                if seen.insert(a.name.clone()) {
                    items.push(item(
                        &a.name,
                        &format!("local in /{}", fn_name),
                        &a.description,
                    ));
                }
            }
            BlockItem::Loop(lp) => collect_assigns(&lp.body, fn_name, seen, items),
            BlockItem::Parallel(par) => collect_assigns(&par.body, fn_name, seen, items),
            _ => {}
        }
    }
}

fn find_enclosing_function(ast: &Source, offset: usize) -> Option<&Function> {
    for top in &ast.items {
        if let TopLevel::Function(f) = top {
            if offset >= f.span.start && offset < f.span.end {
                return Some(f);
            }
        }
    }
    None
}

/// True if the cursor sits inside an unclosed `[` on the current line.
fn in_bracket_context(source: &str, offset: usize) -> bool {
    let bytes = source.as_bytes();
    let mut i = offset;
    while i > 0 {
        let b = bytes[i - 1];
        if b == b'[' {
            return true;
        }
        if b == b']' || b == b'\n' {
            return false;
        }
        i -= 1;
    }
    false
}

fn empty_list() -> Value {
    json!({ "isIncomplete": false, "items": [] })
}

#[cfg(test)]
mod tests {
    use super::*;
    use jaw_parse::parse;

    fn labels(v: &Value) -> Vec<String> {
        v["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|it| it["label"].as_str().unwrap().to_string())
            .collect()
    }

    #[test]
    fn returns_nothing_outside_brackets() {
        let source = "[V] — a vector\n\n/use\n\t[>] V\n";
        let (ast, _) = parse(source);
        // Offset at the bare `V` on line 3 — not inside brackets.
        let offset = source.rfind("] V").unwrap() + 2;
        let result = completion_at(&ast, source, offset);
        assert!(result["items"].as_array().unwrap().is_empty());
    }

    #[test]
    fn offers_enclosing_function_args_and_globals() {
        let source = "[V] — a vector\n[T] — a threshold\n\n/use [X]: a number\n\t[>] [\n";
        let (ast, _) = parse(source);
        // Offset immediately after the trailing `[` on the [>] line.
        let offset = source.rfind('[').unwrap() + 1;
        let result = completion_at(&ast, source, offset);
        let names = labels(&result);
        assert!(names.contains(&"X".to_string()), "missing arg X: {:?}", names);
        assert!(names.contains(&"V".to_string()), "missing global V: {:?}", names);
        assert!(names.contains(&"T".to_string()), "missing global T: {:?}", names);
    }

    #[test]
    fn offers_inline_assigns_from_body() {
        let source = "/process\n[V]: a list\n\t[R]: results\n\t[L]: length\n\t[1] — [\n";
        let (ast, _) = parse(source);
        let offset = source.rfind('[').unwrap() + 1;
        let result = completion_at(&ast, source, offset);
        let names = labels(&result);
        assert!(names.contains(&"R".to_string()), "missing [R]: {:?}", names);
        assert!(names.contains(&"L".to_string()), "missing [L]: {:?}", names);
        assert!(names.contains(&"V".to_string()), "missing arg V: {:?}", names);
    }

    #[test]
    fn does_not_leak_other_functions_locals() {
        let source = "/a [X]: a number\n\t[1] — [Y]: local = [X]\n\n/b [Q]: other\n\t[1] — [\n";
        let (ast, _) = parse(source);
        let offset = source.rfind('[').unwrap() + 1;
        let result = completion_at(&ast, source, offset);
        let names = labels(&result);
        assert!(names.contains(&"Q".to_string()), "expected Q from /b: {:?}", names);
        assert!(!names.contains(&"X".to_string()), "X should not leak from /a: {:?}", names);
    }

    #[test]
    fn partial_prefix_inside_brackets_triggers() {
        // Common case: user is typing `[Us` and completion fires mid-word.
        let source = "[User] — a user\n\n/show\n\t[>] [Us\n";
        let (ast, _) = parse(source);
        let offset = source.rfind("[Us").unwrap() + 3;
        let result = completion_at(&ast, source, offset);
        let names = labels(&result);
        assert!(names.contains(&"User".to_string()), "missing User: {:?}", names);
    }
}
