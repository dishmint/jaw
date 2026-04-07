# Releasing JAW

JAW uses Git tags to trigger releases. Pushing a tag matching `v*` runs the
release workflow at `.github/workflows/release.yml`, which builds the
`jaw-lsp` binaries for macOS, Linux, and Windows, packages the VS Code
extension as a `.vsix`, and attaches everything to a new GitHub Release.

## Cutting a release

From a clean working tree on `main`:

```bash
scripts/release.sh 0.2.0
```

The script will:

1. Validate the version is valid semver
2. Bump the version in `jaw-parse/Cargo.toml`, `jaw-lsp/Cargo.toml`, and `editors/vscode/package.json`
3. Refresh `Cargo.lock` and `editors/vscode/package-lock.json`
4. Commit the bumps as `Release v0.2.0`
5. Create the annotated tag `v0.2.0`

It does **not** push. Review the commit, then:

```bash
git push && git push origin v0.2.0
```

Pushing the tag triggers the release workflow. Check the [Actions tab](https://github.com/dishmint/jaw/actions) for progress, and the [Releases page](https://github.com/dishmint/jaw/releases) for the published artifacts.

## Aborting before push

If something looks wrong before you push:

```bash
git tag -d v0.2.0
git reset --hard HEAD~1
```

## Versioning

JAW follows [semantic versioning](https://semver.org/):

- **Patch** (`0.1.0` → `0.1.1`) — bug fixes, no API or syntax changes
- **Minor** (`0.1.0` → `0.2.0`) — new language features or LSP capabilities, backward compatible
- **Major** (`0.1.0` → `1.0.0`) — breaking changes to the JAW spec or LSP protocol

Pre-releases are supported (e.g. `1.0.0-beta.1`).
