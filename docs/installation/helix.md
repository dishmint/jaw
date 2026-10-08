# Helix Installation

Helix support lives in [`editors/helix/`](../../editors/helix/). It provides:

- Syntax highlighting for `.jaw` files and ` ```jaw ` blocks in Markdown, from the tree-sitter grammar in [`tree-sitter-jaw/`](../../tree-sitter-jaw/)
- `jaw-lsp` diagnostics, hover and go-to-definition

Tested with Helix 25.07.1.

## Install

1. Put `jaw-lsp` on your `PATH`. Use `./scripts/install.sh`, a release download, or `cargo build --release -p jaw-lsp` (see [VS Code installation](vscode.md) for the options). To use a binary that isn't on `PATH`, set `command` in step 2 to its absolute path.

2. Append [`editors/helix/languages.toml`](../../editors/helix/languages.toml) to `~/.config/helix/languages.toml`. Create the file if it doesn't exist.

3. Copy the highlight queries:

   ```sh
   mkdir -p ~/.config/helix/runtime/queries/jaw
   cp editors/helix/queries/jaw/highlights.scm ~/.config/helix/runtime/queries/jaw/
   ```

4. Fetch and build the grammar. This needs a C compiler; on macOS that comes from the Xcode command line tools.

   ```sh
   hx --grammar fetch
   hx --grammar build
   ```

5. Check:

   ```sh
   hx --health jaw
   ```

   Expected: `jaw-lsp` found, `Tree-sitter parser: ✓`, `Highlight queries: ✓`.

## Updating

When `tree-sitter-jaw/` changes, update `rev` in your `languages.toml` to match `editors/helix/languages.toml`, copy `highlights.scm` again, and re-run step 4.

## Troubleshooting

| Symptom | Fix |
|---|---|
| `jaw-lsp` shows ✘ in `hx --health jaw` | It isn't on the `PATH` Helix sees. Set `command` to the absolute path. |
| `Bad CPU type in executable` on macOS | An Intel `hx` or `jaw-lsp` on Apple Silicon. Reinstall Helix with arm64 Homebrew (`/opt/homebrew/bin/brew install helix`) or rebuild `jaw-lsp` with an arm64 toolchain. |
| No colours, parser ✘ | `hx --grammar build` failed, so check that `cc --version` works. |
