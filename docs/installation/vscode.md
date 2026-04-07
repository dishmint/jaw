# VS Code Installation

## Prerequisites

- [Rust toolchain](https://rustup.rs/)
- [Node.js](https://nodejs.org/) (for building the extension)
- VS Code 1.75.0 or later

## 1. Build the language server

```bash
cargo build --release
```

This produces the `jaw-lsp` binary at `target/release/jaw-lsp`.

## 2. Build and install the VS Code extension

```bash
cd editors/vscode
npm install
npm run build
npm run package
```

This produces a `.vsix` file. Install it in VS Code:

```bash
code --install-extension jaw-language-0.1.0.vsix
```

## 3. Configure the LSP path

In VS Code settings, set `jaw.server.path` to the absolute path of the `jaw-lsp` binary:

```json
{
  "jaw.server.path": "/path/to/jaw/target/release/jaw-lsp"
}
```

If `jaw-lsp` is already in your `PATH`, this step is optional.
