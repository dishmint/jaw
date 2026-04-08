use std::collections::HashSet;

use jaw_parse::ast::*;
use jaw_parse::error::{Diagnostic as ParseDiagnostic, Severity};
use serde::Serialize;
use serde_json::{json, Value};

#[derive(Debug, Serialize)]
pub struct LspDiagnostic {
    pub range: LspRange,
    pub severity: u32,
    pub source: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct LspRange {
    pub start: LspPosition,
    pub end: LspPosition,
}

#[derive(Debug, Serialize)]
pub struct LspPosition {
    pub line: u32,
    pub character: u32,
}

/// Convert a byte offset to (line, character) in the source.
fn offset_to_position(source: &str, offset: usize) -> LspPosition {
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

    LspPosition {
        line,
        character: col,
    }
}

/// Convert parse diagnostics to LSP diagnostics.
pub fn to_lsp_diagnostics(source: &str, diags: &[ParseDiagnostic]) -> Vec<LspDiagnostic> {
    diags
        .iter()
        .map(|d| {
            let start = offset_to_position(source, d.span.start);
            let end = offset_to_position(source, d.span.end);
            LspDiagnostic {
                range: LspRange { start, end },
                severity: match d.severity {
                    Severity::Error => 1,
                    Severity::Warning => 2,
                },
                source: "jaw".to_string(),
                message: d.message.clone(),
            }
        })
        .collect()
}

/// Collect all defined function names from the AST.
fn collect_function_names(ast: &Source) -> HashSet<String> {
    let mut names = HashSet::new();
    for item in &ast.items {
        if let TopLevel::Function(f) = item {
            names.insert(f.name.clone());
        }
    }
    names
}

/// Find bare identifiers in text that match defined function names.
/// Returns (byte_offset_in_source, identifier) pairs.
fn find_bare_function_refs(source: &str, text: &str, text_offset: usize, func_names: &HashSet<String>) -> Vec<(usize, String)> {
    let mut results = Vec::new();
    let bytes = text.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        // Skip if preceded by '/' — it's already a proper function ref
        if i > 0 && bytes[i - 1] == b'/' {
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
            continue;
        }

        if bytes[i].is_ascii_alphabetic() || bytes[i] == b'_' {
            let start = i;
            let mut ident = String::new();
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                ident.push(bytes[i] as char);
                i += 1;
            }

            // Check if preceded by '/' in the source (not just text)
            let source_pos = text_offset + start;
            let preceded_by_slash = source_pos > 0
                && source.as_bytes().get(source_pos - 1) == Some(&b'/');

            if !preceded_by_slash && func_names.contains(&ident) {
                results.push((source_pos, ident));
            }
        } else {
            i += 1;
        }
    }

    results
}

/// Scan the AST for bare function name references and produce warnings.
pub fn check_bare_function_refs(ast: &Source, source: &str) -> Vec<LspDiagnostic> {
    let func_names = collect_function_names(ast);
    if func_names.is_empty() {
        return Vec::new();
    }

    let mut warnings = Vec::new();

    for item in &ast.items {
        match item {
            TopLevel::Function(f) => {
                for arg in &f.args {
                    check_inline_assign_for_bare_refs(arg, source, &func_names, &mut warnings);
                }
                check_block_for_bare_refs(&f.body, source, &func_names, &mut warnings);
            }
            TopLevel::Step(step) => {
                check_step_for_bare_refs(step, source, &func_names, &mut warnings);
            }
            _ => {}
        }
    }

    warnings
}

fn check_step_for_bare_refs(
    step: &Step,
    source: &str,
    func_names: &HashSet<String>,
    warnings: &mut Vec<LspDiagnostic>,
) {
    match &step.expression {
        Expression::Code(s) => {
            check_text_for_bare_refs(source, s, step.span.start, func_names, warnings);
        }
        Expression::Conditional(c) => {
            for branch in &c.branches {
                check_text_for_bare_refs(
                    source, &branch.condition, step.span.start, func_names, warnings,
                );
                check_text_for_bare_refs(
                    source, &branch.consequence, step.span.start, func_names, warnings,
                );
            }
            if let Some(ref else_text) = c.else_branch {
                check_text_for_bare_refs(
                    source, else_text, step.span.start, func_names, warnings,
                );
            }
        }
    }
}

fn check_block_for_bare_refs(
    block: &CodeBlock,
    source: &str,
    func_names: &HashSet<String>,
    warnings: &mut Vec<LspDiagnostic>,
) {
    for item in &block.items {
        match item {
            BlockItem::Step(step) => {
                let text = match &step.expression {
                    Expression::Code(s) => s.as_str(),
                    Expression::Conditional(c) => {
                        // Check condition and consequence text in each branch
                        for branch in &c.branches {
                            check_text_for_bare_refs(
                                source, &branch.condition, step.span.start, func_names, warnings,
                            );
                            check_text_for_bare_refs(
                                source, &branch.consequence, step.span.start, func_names, warnings,
                            );
                        }
                        if let Some(ref else_text) = c.else_branch {
                            check_text_for_bare_refs(
                                source, else_text, step.span.start, func_names, warnings,
                            );
                        }
                        continue;
                    }
                };
                check_text_for_bare_refs(source, text, step.span.start, func_names, warnings);
            }
            BlockItem::Loop(lp) => {
                check_block_for_bare_refs(&lp.body, source, func_names, warnings);
            }
            BlockItem::Parallel(par) => {
                check_block_for_bare_refs(&par.body, source, func_names, warnings);
            }
            BlockItem::InlineAssign(assign) => {
                check_inline_assign_for_bare_refs(assign, source, func_names, warnings);
            }
            _ => {}
        }
    }
}

fn check_inline_assign_for_bare_refs(
    assign: &InlineAssign,
    source: &str,
    func_names: &HashSet<String>,
    warnings: &mut Vec<LspDiagnostic>,
) {
    if let Some(value) = &assign.value {
        check_text_for_bare_refs(source, value, assign.span.start, func_names, warnings);
    }
}

fn check_text_for_bare_refs(
    source: &str,
    text: &str,
    context_offset: usize,
    func_names: &HashSet<String>,
    warnings: &mut Vec<LspDiagnostic>,
) {
    // Find the actual position of the expression text within the source
    let text_offset = source[context_offset..]
        .find(text)
        .map(|pos| context_offset + pos)
        .unwrap_or(context_offset);
    let refs = find_bare_function_refs(source, text, text_offset, func_names);
    for (offset, name) in refs {
        let start = offset_to_position(source, offset);
        let end = offset_to_position(source, offset + name.len());
        warnings.push(LspDiagnostic {
            range: LspRange { start, end },
            severity: 2, // Warning
            source: "jaw".to_string(),
            message: format!("Did you mean `/{}`?", name),
        });
    }
}

/// Build the publishDiagnostics notification params.
pub fn publish_diagnostics_params(uri: &str, source: &str, ast: &Source, diags: &[ParseDiagnostic]) -> Value {
    let mut lsp_diags = to_lsp_diagnostics(source, diags);
    lsp_diags.extend(check_bare_function_refs(ast, source));
    json!({
        "uri": uri,
        "diagnostics": lsp_diags
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use jaw_parse::parse;

    #[test]
    fn warns_on_bare_function_ref_in_inline_assignment_value() {
        let source = "/Length [X]: a vector\n    [>] 0\n\n/Caller\n    [F]: forward end = Length[ [V] ] - 1\n";
        let (ast, _diags) = parse(source);
        let warnings = check_bare_function_refs(&ast, source);
        let messages: Vec<&str> = warnings.iter().map(|w| w.message.as_str()).collect();
        assert!(
            messages.iter().any(|m| m.contains("`/Length`")),
            "expected a `did you mean /Length?` warning, got: {:?}",
            messages
        );
    }

    #[test]
    fn warns_on_bare_function_ref_in_function_arg_default() {
        let source = "/Length [X]: a vector\n    [>] 0\n\n/Caller [N]: count = Length[ [V] ]\n    [>] [N]\n";
        let (ast, _diags) = parse(source);
        let warnings = check_bare_function_refs(&ast, source);
        let messages: Vec<&str> = warnings.iter().map(|w| w.message.as_str()).collect();
        assert!(
            messages.iter().any(|m| m.contains("`/Length`")),
            "expected a `did you mean /Length?` warning on arg default, got: {:?}",
            messages
        );
    }
}
