# JAW Quick Look (macOS)

Makes `.jaw` a **recognized file type** in Finder and adds a **syntax-highlighted
Quick Look preview** (press <kbd>Space</kbd> on a `.jaw` file). Resolves
[#63](https://github.com/dishmint/jaw/issues/63).

It ships as a small macOS app (`JAW Quick Look.app`) that contains a Quick Look
preview extension. The app has to exist because macOS only registers a file type
(a Uniform Type Identifier) and loads a preview extension when they're declared by
an installed application bundle — there's no standalone "register this extension"
on modern macOS.

| Piece | Role |
| --- | --- |
| `JAWQuickLook` (app) | Exports the `com.dishmint.jaw.source` UTI for the `.jaw` extension and hosts the preview extension. |
| `JAWQuickLookExtension` (app extension) | A `QLPreviewingController` that renders the file as highlighted HTML in a `WKWebView`. |
| `JAWHighlighter.swift` | Dependency-free, line-oriented JAW → HTML highlighter. Light + dark via `prefers-color-scheme`. |

Because the UTI conforms to `public.source-code`, even before the extension loads
macOS will already preview `.jaw` files as plain text instead of "unknown file".

## Requirements

- macOS 12 or later
- Xcode 14+ (command-line tools alone are not enough — this builds an app bundle)
- [XcodeGen](https://github.com/yonom/XcodeGen): `brew install xcodegen`

The Xcode project is **generated** from `project.yml` rather than checked in, so the
repo stays free of a giant `project.pbxproj`. Regenerate it any time the file set
changes.

## Build

```bash
cd editors/macos
xcodegen generate            # writes JAWQuickLook.xcodeproj
open JAWQuickLook.xcodeproj   # then ⌘R, or build from the CLI below
```

Command-line build:

```bash
cd editors/macos
xcodegen generate
xcodebuild -project JAWQuickLook.xcodeproj \
           -scheme JAWQuickLook \
           -configuration Release \
           -derivedDataPath build
# Result: build/Build/Products/Release/JAW Quick Look.app
```

### Signing

Set your Apple Developer **Team ID** so the app and its embedded extension share a
signing identity — either edit `DEVELOPMENT_TEAM` in `project.yml` (then
re-run `xcodegen generate`), or pick a team in Xcode's *Signing & Capabilities*
tab. For purely local testing, Xcode's automatic "Sign to Run Locally" works too.

## Install

macOS discovers extensions from apps in a launchable location. Move the built app
into `/Applications` (or `~/Applications`) and launch it once:

```bash
cp -R "build/Build/Products/Release/JAW Quick Look.app" /Applications/
open "/Applications/JAW Quick Look.app"
```

Launching registers the UTI and the extension with Launch Services. You can quit
the window afterward — the preview keeps working.

If macOS doesn't pick it up immediately, nudge Launch Services and the Quick Look
daemon:

```bash
/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister \
  -f "/Applications/JAW Quick Look.app"
qlmanage -r && qlmanage -r cache
```

> **Gatekeeper note:** an unsigned/un-notarized app downloaded from the internet
> gets quarantined. If you distribute the built `.app`, the same notarization work
> tracked in [#24](https://github.com/dishmint/jaw/issues/24) applies, or users can
> clear the flag with `xattr -dr com.apple.quarantine "/Applications/JAW Quick Look.app"`.

## Test

In Finder, select any file from `../../samples/` (e.g. `full.jaw`, `notes.jaw`) and
press <kbd>Space</kbd>.

From the command line, render a preview without Finder:

```bash
qlmanage -p ../../samples/full.jaw          # opens a preview window
qlmanage -m plugins | grep -i jaw           # confirm the generator is registered
```

Tail the extension's logs while previewing:

```bash
log stream --predicate 'subsystem CONTAINS "quicklook"' --info
```

## How highlighting works

`JAWHighlighter` tokenizes each line with a single combined regular expression and
emits `<span>`s with CSS classes:

| Class | JAW construct |
| --- | --- |
| `marker` | `[^] [*] [•] [!] [>] [~] [&] [+] [-]` |
| `step` | `[1]`, `[2]`, … (all-digit brackets) |
| `var` | `[V]`, `[ID]` variable refs |
| `fn` | `/Name` function refs |
| `deco` | `#name` / `#name:value` decorators |
| `sep` | the `—` em dash |
| `op` | operators and `@` array access |
| `num` | numeric literals |

Line-level classes (`note`, `log`, `comment`) carry the spec's emphasis — important
notes render red + bold, comments muted/italic, the `[•]` log marker amber. The
palette tracks the VS Code extension (`editors/vscode/src/extension.ts`).

It's intentionally approximate: a preview only needs to read well, and the
highlighter never fails — anything it can't classify falls through as plain text.
The authoritative grammar remains `jaw-grammar.md` and the parser in `jaw-parse`.
