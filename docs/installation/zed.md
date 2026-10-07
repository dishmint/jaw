# Zed Installation

The Zed extension lives in [`editors/zed/`](../../editors/zed/). It provides:

- Syntax highlighting for `.jaw` files and ` ```jaw ` blocks in Markdown, from the tree-sitter grammar in [`tree-sitter-jaw/`](../../tree-sitter-jaw/)
- `jaw-lsp` diagnostics, hover and go-to-definition
- Outline entries for `/Function` definitions

It is not in the Zed extension registry yet, so install it as a dev extension.

## Install

1. Install Rust with [rustup](https://rustup.rs/). Zed needs it to compile the extension to WebAssembly.
2. Clone the repo:

   ```sh
   git clone https://github.com/dishmint/jaw
   ```

3. In Zed, open the command palette and run **zed: install dev extension**, then pick `jaw/editors/zed`.

Zed fetches the grammar at the commit pinned in `editors/zed/extension.toml` and compiles it. Open any `.jaw` file to confirm highlighting.

## Language server

The extension looks for `jaw-lsp` in this order:

1. `lsp.jaw-lsp.binary.path` in Zed settings
2. `jaw-lsp` on your `PATH`
3. The latest [GitHub release](https://github.com/dishmint/jaw/releases), downloaded automatically (macOS, Linux x86_64, Windows x86_64)

To point at a local build:

```json
{
  "lsp": {
    "jaw-lsp": {
      "binary": { "path": "/absolute/path/to/target/release/jaw-lsp" }
    }
  }
}
```

## Developing the grammar

Zed compiles the grammar from a git commit, not from your working tree. To try local grammar changes, temporarily point `extension.toml` at your clone:

```toml
[grammars.jaw]
repository = "file:///absolute/path/to/jaw"
rev = "<a local commit containing your changes>"
path = "tree-sitter-jaw"
```

Then run **zed: rebuild dev extension**. See [`tree-sitter-jaw/README.md`](../../tree-sitter-jaw/README.md) for regenerating the parser.
