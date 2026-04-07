# VS Code Installation

## Option A: Prebuilt download (recommended)

1. Go to the [Releases page](https://github.com/dishmint/jaw/releases) and download:
   - The `.vsix` extension file (e.g. `jaw-language-0.1.0.vsix`)
   - The `jaw-lsp` archive for your platform:
     - macOS (Apple Silicon): `jaw-lsp-aarch64-apple-darwin.tar.gz`
     - macOS (Intel): `jaw-lsp-x86_64-apple-darwin.tar.gz`
     - Linux: `jaw-lsp-x86_64-unknown-linux-gnu.tar.gz`
     - Windows: `jaw-lsp-x86_64-pc-windows-msvc.zip`

2. Extract the `jaw-lsp` archive and move the binary somewhere stable (e.g. `~/.local/bin/jaw-lsp` or `/usr/local/bin/jaw-lsp`).

3. Install the extension:

   ```bash
   code --install-extension jaw-language-0.1.0.vsix
   ```

4. In VS Code settings, set `jaw.server.path` to the absolute path of the `jaw-lsp` binary:

   ```json
   {
     "jaw.server.path": "/absolute/path/to/jaw-lsp"
   }
   ```

   If `jaw-lsp` is on your `PATH`, this step is optional.

## Option B: Build from source

### Prerequisites

- [Rust toolchain](https://rustup.rs/)
- [Node.js](https://nodejs.org/) (for building the extension)
- VS Code 1.75.0 or later

### 1. Build the language server

```bash
cargo build --release
```

This produces the `jaw-lsp` binary at `target/release/jaw-lsp`.

### 2. Build and package the VS Code extension

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

### 3. Configure the LSP path

In VS Code settings, set `jaw.server.path` to the absolute path of the `jaw-lsp` binary:

```json
{
  "jaw.server.path": "/path/to/jaw/target/release/jaw-lsp"
}
```

If `jaw-lsp` is already in your `PATH`, this step is optional.
