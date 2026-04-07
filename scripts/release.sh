#!/usr/bin/env bash
#
# Prepare a JAW release: bump versions, commit, and tag.
#
# Usage:
#   scripts/release.sh <version>
#
# Example:
#   scripts/release.sh 0.2.0
#
# This script does NOT push. After it finishes, review the commit and tag, then:
#   git push && git push origin v<version>
# Pushing the tag triggers the release workflow which builds artifacts and
# creates the GitHub Release.

set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "Usage: $0 <version>" >&2
  echo "Example: $0 0.2.0" >&2
  exit 1
fi

VERSION="$1"

# Validate semver (MAJOR.MINOR.PATCH with optional -prerelease)
if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$ ]]; then
  echo "Error: '$VERSION' is not a valid semver version (e.g. 0.2.0 or 1.0.0-beta.1)" >&2
  exit 1
fi

TAG="v$VERSION"

# Run from repo root
REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "$REPO_ROOT"

# Refuse to release with a dirty working tree
if ! git diff --quiet || ! git diff --cached --quiet; then
  echo "Error: working tree is dirty. Commit or stash changes first." >&2
  exit 1
fi

# Refuse if the tag already exists
if git rev-parse "$TAG" >/dev/null 2>&1; then
  echo "Error: tag $TAG already exists." >&2
  exit 1
fi

echo "==> Bumping versions to $VERSION"

# Replace the first matching line in a file. Portable across BSD and GNU.
bump_first() {
  local file="$1" pattern="$2" replacement="$3"
  awk -v pat="$pattern" -v rep="$replacement" '
    BEGIN { done = 0 }
    !done && $0 ~ pat { print rep; done = 1; next }
    { print }
  ' "$file" > "$file.new"
  mv "$file.new" "$file"
}

# Cargo crates: first `version = "..."` line
bump_first jaw-parse/Cargo.toml '^version = ".*"$' "version = \"$VERSION\""
bump_first jaw-lsp/Cargo.toml '^version = ".*"$' "version = \"$VERSION\""

# VS Code extension: first `"version": "..."` line (the top-level field)
bump_first editors/vscode/package.json '"version": ".*"' "  \"version\": \"$VERSION\","

echo "==> Updating Cargo.lock"
cargo check --quiet

echo "==> Updating editors/vscode/package-lock.json"
(cd editors/vscode && npm install --silent)

if git diff --quiet; then
  echo "==> Version is already $VERSION; nothing to commit"
  COMMIT_SHA="$(git rev-parse --short HEAD)"
else
  echo "==> Staging and committing"
  git add \
    jaw-parse/Cargo.toml \
    jaw-lsp/Cargo.toml \
    Cargo.lock \
    editors/vscode/package.json \
    editors/vscode/package-lock.json
  git commit -m "Release $TAG"
  COMMIT_SHA="$(git rev-parse --short HEAD)"
fi

echo "==> Tagging $TAG"
git tag -a "$TAG" -m "Release $TAG"

cat <<EOF

Release prepared:
  - Commit: $COMMIT_SHA
  - Tag:    $TAG

Next steps:
  1. Review the commit:    git show HEAD
  2. Push commit and tag:  git push && git push origin $TAG

Pushing the tag triggers the release workflow at .github/workflows/release.yml,
which builds the jaw-lsp binaries and VSIX and creates the GitHub Release.

To abort instead:
  git tag -d $TAG
  # If a release commit was created, also undo it:
  #   git reset --hard HEAD~1
EOF
