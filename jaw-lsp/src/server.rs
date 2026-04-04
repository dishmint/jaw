use std::collections::HashMap;
use std::io::{BufRead, Write};

use serde_json::{json, Value};

use crate::diagnostics::publish_diagnostics_params;
use crate::goto::goto_definition;
use crate::hover::hover_at;
use crate::rpc::{self, RpcMessage, RpcNotification, RpcResponse};

pub struct Server {
    /// Open document contents, keyed by URI.
    documents: HashMap<String, String>,
    /// Parsed ASTs, keyed by URI.
    asts: HashMap<String, jaw_parse::ast::Source>,
    initialized: bool,
    shutdown: bool,
}

impl Server {
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
            asts: HashMap::new(),
            initialized: false,
            shutdown: false,
        }
    }

    pub fn run(&mut self, reader: &mut impl BufRead, writer: &mut impl Write) {
        loop {
            let msg = match rpc::read_message(reader) {
                Ok(Some(msg)) => msg,
                Ok(None) => break, // EOF
                Err(e) => {
                    eprintln!("jaw-lsp: read error: {}", e);
                    break;
                }
            };

            let should_exit = self.handle_message(msg, writer);
            if should_exit {
                break;
            }
        }
    }

    fn handle_message(&mut self, msg: RpcMessage, writer: &mut impl Write) -> bool {
        let method = match msg.method {
            Some(ref m) => m.as_str(),
            None => return false,
        };

        match method {
            "initialize" => {
                let result = json!({
                    "capabilities": {
                        "textDocumentSync": 1,
                        "hoverProvider": true,
                        "definitionProvider": true
                    },
                    "serverInfo": {
                        "name": "jaw-lsp",
                        "version": "0.1.0"
                    }
                });
                let resp = RpcResponse::success(msg.id, result);
                let _ = rpc::write_message(writer, &resp);
                self.initialized = true;
            }

            "initialized" => {
                // Client acknowledgement, nothing to do
            }

            "shutdown" => {
                self.shutdown = true;
                let resp = RpcResponse::success(msg.id, Value::Null);
                let _ = rpc::write_message(writer, &resp);
            }

            "exit" => {
                return true;
            }

            "textDocument/didOpen" => {
                if let Some(params) = msg.params {
                    if let Some(doc) = params.get("textDocument") {
                        let uri = doc["uri"].as_str().unwrap_or_default().to_string();
                        let text = doc["text"].as_str().unwrap_or_default().to_string();
                        self.update_document(&uri, &text, writer);
                    }
                }
            }

            "textDocument/didChange" => {
                if let Some(params) = msg.params {
                    let uri = params["textDocument"]["uri"]
                        .as_str()
                        .unwrap_or_default()
                        .to_string();
                    // Full sync — take last content change
                    if let Some(changes) = params["contentChanges"].as_array() {
                        if let Some(last) = changes.last() {
                            let text = last["text"].as_str().unwrap_or_default().to_string();
                            self.update_document(&uri, &text, writer);
                        }
                    }
                }
            }

            "textDocument/hover" => {
                let result = self.handle_hover(&msg.params);
                let resp = RpcResponse::success(msg.id, result.unwrap_or(Value::Null));
                let _ = rpc::write_message(writer, &resp);
            }

            "textDocument/definition" => {
                let result = self.handle_definition(&msg.params);
                let resp = RpcResponse::success(msg.id, result.unwrap_or(Value::Null));
                let _ = rpc::write_message(writer, &resp);
            }

            "textDocument/didClose" => {
                if let Some(params) = msg.params {
                    let uri = params["textDocument"]["uri"]
                        .as_str()
                        .unwrap_or_default()
                        .to_string();
                    self.documents.remove(&uri);
                    self.asts.remove(&uri);
                }
            }

            _ => {
                // Unknown method — respond with error if it has an id (request)
                if msg.id.is_some() {
                    let resp = RpcResponse::error(msg.id, -32601, "method not found");
                    let _ = rpc::write_message(writer, &resp);
                }
            }
        }

        false
    }

    fn update_document(&mut self, uri: &str, text: &str, writer: &mut impl Write) {
        self.documents.insert(uri.to_string(), text.to_string());

        // Parse and publish diagnostics
        let (ast, diags) = jaw_parse::parse(text);
        self.asts.insert(uri.to_string(), ast);

        let params = publish_diagnostics_params(uri, text, &diags);
        let notification = RpcNotification::new("textDocument/publishDiagnostics", params);
        let _ = rpc::write_message(writer, &notification);
    }

    fn handle_hover(&self, params: &Option<Value>) -> Option<Value> {
        let params = params.as_ref()?;
        let uri = params["textDocument"]["uri"].as_str()?;
        let line = params["position"]["line"].as_u64()? as usize;
        let character = params["position"]["character"].as_u64()? as usize;

        let source = self.documents.get(uri)?;
        let ast = self.asts.get(uri)?;
        let offset = position_to_offset(source, line, character)?;

        hover_at(ast, source, offset)
    }

    fn handle_definition(&self, params: &Option<Value>) -> Option<Value> {
        let params = params.as_ref()?;
        let uri = params["textDocument"]["uri"].as_str()?;
        let line = params["position"]["line"].as_u64()? as usize;
        let character = params["position"]["character"].as_u64()? as usize;

        let source = self.documents.get(uri)?;
        let ast = self.asts.get(uri)?;
        let offset = position_to_offset(source, line, character)?;

        let mut result = goto_definition(ast, source, offset)?;
        // Add uri to the location
        if let Some(obj) = result.as_object_mut() {
            obj.insert("uri".to_string(), json!(uri));
        }
        Some(result)
    }
}

/// Convert (line, character) to a byte offset in the source.
fn position_to_offset(source: &str, line: usize, character: usize) -> Option<usize> {
    let mut current_line = 0;
    let mut current_col = 0;

    for (i, ch) in source.char_indices() {
        if current_line == line && current_col == character {
            return Some(i);
        }
        if ch == '\n' {
            if current_line == line {
                return Some(i); // Past end of line
            }
            current_line += 1;
            current_col = 0;
        } else {
            current_col += 1;
        }
    }

    if current_line == line {
        Some(source.len())
    } else {
        None
    }
}
