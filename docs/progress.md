# JAW Progress

## Completed

### v1 Syntax Spec (PR #2, merged)
- Defined all JAW constructs: variables, functions, steps, comments, logging, returns, conditionals, loops, parallel ops, decorators, array access, operators
- Established conventions: `[~]` unified loop marker, `[&]` parallel, `#` decorators, `@` array access, `?`/`|` conditionals, `[+]`/`[-]` complex branches, `<<` append
- Function references use `/` prefix everywhere
- Multi-line comments and logs via continuation
- Log titles (ending in `:`) and log-level decorators (`#error`, `#warn`)
- Formal grammar in `jaw-grammar.md`

### Grammar Parser & LSP (PR #4, merged)
- **TextMate grammar** (`editors/vscode/syntaxes/jaw.tmLanguage.json`) — syntax highlighting for all constructs
- **Markdown injection** — ` ```jaw ` code blocks highlighted in markdown
- **Rust parser** (`jaw-parse/`) — hand-written lexer + recursive descent parser, 18 tests
- **LSP server** (`jaw-lsp/`) — diagnostics, hover, go-to-definition over JSON-RPC
- **VS Code extension** — language registration, LSP client, bold/italic token styles
- Only dependency: `serde`/`serde_json`

### README Restructure (PR #5, open)
- Simplified layout with inline syntax descriptions
- Single example screenshot
- Glossary (`docs/glossary.md`) with all markers and terms
- TextMate grammar fixes: multi-line comments/logs, log titles, function refs, variable refs in comments, `@` and `=` operator coloring

## Open

### LSP Enhancements
- "Did you mean `/func`?" warning when a bare identifier matches a defined function name
- Quick-fix code action to prepend `/`

### Future Considerations
- tree-sitter grammar for Neovim/Helix/Zed highlighting
- Linguist submission for GitHub ` ```jaw ` highlighting
- VS Code color theme for shipping bold/italic styles without programmatic settings
- Semantic tokens from LSP for richer highlighting
