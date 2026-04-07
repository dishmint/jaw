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
#
# Requirements: curl, tar, and (for the extension) the `code` CLI on PATH.

set -euo pipefail

REPO="dishmint/jaw"
INSTALL_DIR="${JAW_INSTALL_DIR:-$HOME/.local/bin}"

# --- detect platform ---------------------------------------------------------
uname_s="$(uname -s)"
uname_m="$(uname -m)"

case "$uname_s" in
  Darwin)
    case "$uname_m" in
      arm64)  TARGET="aarch64-apple-darwin" ;;
      x86_64) TARGET="x86_64-apple-darwin" ;;
      *) echo "Unsupported macOS arch: $uname_m" >&2; exit 1 ;;
    esac
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
