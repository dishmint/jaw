#!/usr/bin/env bash
#
# Install (or update) JAW from the latest GitHub release.
#
# Downloads the jaw-lsp binary for the current platform and the VS Code
# extension VSIX, installs jaw-lsp to ~/.local/bin, and installs the VSIX
# via the `code` CLI.
#
# Usage:
#   scripts/install.sh                # latest release
#   scripts/install.sh v0.1.1         # specific tag
#   scripts/install.sh --from-source  # build local working tree and install
#
# Requirements: curl, tar, and (for the extension) the `code` CLI on PATH.
# --from-source additionally needs cargo.

set -euo pipefail

REPO="dishmint/jaw"
INSTALL_DIR="${JAW_INSTALL_DIR:-$HOME/.local/bin}"

# --- from-source mode --------------------------------------------------------
if [[ "${1:-}" == "--from-source" ]]; then
  REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
  if [[ ! -f "$REPO_ROOT/Cargo.toml" ]]; then
    echo "Could not find Cargo.toml at $REPO_ROOT" >&2
    exit 1
  fi

  CARGO="$(command -v cargo || true)"
  if [[ -z "$CARGO" && -x "$HOME/.cargo/bin/cargo" ]]; then
    CARGO="$HOME/.cargo/bin/cargo"
  fi
  if [[ -z "$CARGO" ]]; then
    echo "cargo not found on PATH or at ~/.cargo/bin/cargo" >&2
    exit 1
  fi

  echo "==> Building jaw-lsp from $REPO_ROOT"
  (cd "$REPO_ROOT" && "$CARGO" build --release -p jaw-lsp)

  BIN="$REPO_ROOT/target/release/jaw-lsp"
  if [[ ! -x "$BIN" ]]; then
    echo "Build did not produce $BIN" >&2
    exit 1
  fi

  mkdir -p "$INSTALL_DIR"
  install -m 0755 "$BIN" "$INSTALL_DIR/jaw-lsp"

  if [[ "$(uname -s)" == "Darwin" ]]; then
    xattr -d com.apple.quarantine "$INSTALL_DIR/jaw-lsp" 2>/dev/null || true
  fi

  echo "==> Installed jaw-lsp to $INSTALL_DIR/jaw-lsp"
  ls -la "$INSTALL_DIR/jaw-lsp"

  cat <<EOF

Done. Make sure $INSTALL_DIR is on your PATH, then set in VS Code:

  "jaw.server.path": "$INSTALL_DIR/jaw-lsp"

Restart VS Code (or run "Developer: Restart Extension Host") to pick
up the new binary. The VSIX is not rebuilt by --from-source; if you've
edited the extension, rebuild it manually from editors/vscode/.
EOF
  exit 0
fi

# --- detect platform ---------------------------------------------------------
uname_s="$(uname -s)"
uname_m="$(uname -m)"

case "$uname_s" in
  Darwin)
    # `uname -m` reports the *process* arch, which inherits from the parent
    # and reads as x86_64 under Rosetta even on Apple Silicon. Use sysctl to
    # query the host arch directly.
    if [[ "$(sysctl -n hw.optional.arm64 2>/dev/null)" == "1" ]]; then
      TARGET="aarch64-apple-darwin"
    else
      TARGET="x86_64-apple-darwin"
    fi
    ARCHIVE_EXT="tar.gz"
    ;;
  Linux)
    case "$uname_m" in
      x86_64) TARGET="x86_64-unknown-linux-gnu" ;;
      *) echo "Unsupported Linux arch: $uname_m" >&2; exit 1 ;;
    esac
    ARCHIVE_EXT="tar.gz"
    ;;
  *)
    echo "Unsupported OS: $uname_s (use the Windows release manually)" >&2
    exit 1
    ;;
esac

# --- resolve release tag -----------------------------------------------------
TAG="${1:-}"
if [[ -z "$TAG" ]]; then
  echo "==> Fetching latest release tag"
  TAG="$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" \
    | grep -m1 '"tag_name"' \
    | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/')"
  if [[ -z "$TAG" ]]; then
    echo "Failed to resolve latest release tag" >&2
    exit 1
  fi
fi
echo "==> Using release $TAG"

BASE_URL="https://github.com/$REPO/releases/download/$TAG"
LSP_ARCHIVE="jaw-lsp-$TARGET.$ARCHIVE_EXT"

# --- download into a temp dir ------------------------------------------------
tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

echo "==> Downloading $LSP_ARCHIVE"
curl -fsSL -o "$tmpdir/$LSP_ARCHIVE" "$BASE_URL/$LSP_ARCHIVE"

echo "==> Extracting jaw-lsp"
tar -xzf "$tmpdir/$LSP_ARCHIVE" -C "$tmpdir"

mkdir -p "$INSTALL_DIR"
install -m 0755 "$tmpdir/jaw-lsp" "$INSTALL_DIR/jaw-lsp"

# Strip macOS quarantine attribute (release binaries are unsigned).
if [[ "$uname_s" == "Darwin" ]]; then
  xattr -d com.apple.quarantine "$INSTALL_DIR/jaw-lsp" 2>/dev/null || true
fi

echo "==> Installed jaw-lsp to $INSTALL_DIR/jaw-lsp"
ls -la "$INSTALL_DIR/jaw-lsp"

# --- install the VS Code extension -------------------------------------------
# The VSIX file name is published as jaw-language-<version>.vsix where <version>
# is the tag without the leading "v".
VERSION="${TAG#v}"
VSIX_FILE="jaw-language-$VERSION.vsix"

echo "==> Downloading $VSIX_FILE"
curl -fsSL -o "$tmpdir/$VSIX_FILE" "$BASE_URL/$VSIX_FILE"

if command -v code >/dev/null 2>&1; then
  echo "==> Installing VS Code extension"
  code --install-extension "$tmpdir/$VSIX_FILE" --force
else
  echo "==> 'code' CLI not found; skipping VS Code extension install."
  echo "    Install it manually from: $tmpdir/$VSIX_FILE"
  echo "    (or re-run after enabling 'Shell Command: Install code command in PATH')"
fi

cat <<EOF

Done. Make sure $INSTALL_DIR is on your PATH, then set in VS Code:

  "jaw.server.path": "$INSTALL_DIR/jaw-lsp"

Reload VS Code to pick up the new extension and LSP binary.
EOF
