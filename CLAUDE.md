# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What is JAW?

JAW (Just A Word) is a pseudocode and commenting language. The repo contains the v1 spec, a Rust parser/LSP, and a VS Code extension for syntax highlighting.

## Build & Test

Cargo is not in the default PATH. Use the full path:

```bash
# Build
~/.cargo/bin/cargo build
~/.cargo/bin/cargo build --release

# Test (18 tests: lexer + parser)
~/.cargo/bin/cargo test

# Run a single test
~/.cargo/bin/cargo test -p jaw-parse test_parse_variable

# VS Code extension (from editors/vscode/)
npm install && npm run build
```

## Architecture

Three layers:

- **jaw-parse** — Hand-written lexer + recursive descent parser. Entry point: `pub fn parse(source: &str) -> (Source, Vec<Diagnostic>)` in `lib.rs`. Lexer handles bracket lookahead to distinguish markers (`[~]`, `[&]`, `[^]`, etc.) from variables (`[V]`) and steps (`[1]`). Parser uses source-span-based text extraction (not token reconstruction) for accurate content.

- **jaw-lsp** — LSP server over JSON-RPC on stdin/stdout. Hand-written transport (`rpc.rs`), no framework. Provides diagnostics (parse errors + "did you mean `/func`?" warnings for bare function refs), hover, and go-to-definition. Only dependency: `serde`/`serde_json`.

- **editors/vscode** — VS Code extension with TextMate grammar (`jaw.tmLanguage.json`), markdown injection for ` ```jaw ` blocks, and LSP client that spawns `jaw-lsp`. Extension also programmatically sets bold/italic token styles on activation since `configurationDefaults` can't set `editor.tokenColorCustomizations`.

## Key Conventions

- The em dash `—` (U+2014, not hyphens) separates markers from content
- Function references always use `/` prefix: `/Add`, `/Process`
- Function names use title case: `/HandlePositive`, not `/handle_positive`
- Variable refs are bracket-enclosed: `[V]`, `[P]`
- Function calls include explicit args: `/Classify[ [X], [T] ]`
- The formal grammar lives in `jaw-grammar.md` and should stay in sync with the parser

## Spec Reference

All bracket markers: `[^]` code comment, `[*]` general comment, `[!]` log, `[>]` return, `[~]` loop, `[&]` parallel, `[+]` true branch, `[-]` false branch, `[N]` step, `[ID]` variable. Decorators: `#name` or `#name:value`. Array access: `[V]@[P]`. See `docs/glossary.md` for the full reference.

## Testing the VS Code Extension

From `editors/vscode/`: press F5 to launch Extension Development Host. Open `.jaw` files from `samples/` or `docs/examples/`. For LSP testing, set `jaw.server.path` in VS Code settings to the built binary path (e.g., the full path to `target/release/jaw-lsp`). LSP test files are in `samples/lsp-test/`.
