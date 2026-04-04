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

/// Build the publishDiagnostics notification params.
pub fn publish_diagnostics_params(uri: &str, source: &str, diags: &[ParseDiagnostic]) -> Value {
    let lsp_diags = to_lsp_diagnostics(source, diags);
    json!({
        "uri": uri,
        "diagnostics": lsp_diags
    })
}
