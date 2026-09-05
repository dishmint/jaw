# macOS Quick Look Installation

`JAW Quick Look.app` makes `.jaw` a recognized file type in Finder and adds a
syntax-highlighted Quick Look preview (press <kbd>Space</kbd> on a `.jaw` file).
It is a small app that exists only to host the preview extension; macOS loads
extensions from installed app bundles and nothing else.

Requires macOS 12 or later.

## Option A: Prebuilt download

1. Go to the [Releases page](https://github.com/dishmint/jaw/releases) and
   download `jaw-quicklook-macos.zip`.

2. Unzip it and move `JAW Quick Look.app` into `/Applications` (or
   `~/Applications`).

3. Open it once. The release build is ad-hoc signed and not notarized, so
   Gatekeeper blocks the first launch:

   - **macOS 15 (Sequoia) and later:** the first attempt is refused. Open
     *System Settings → Privacy & Security*, scroll to the message about
     "JAW Quick Look", click **Open Anyway**, then launch it again.
   - **macOS 14 and earlier:** right-click (or Control-click) the app in
     Finder, choose **Open**, and confirm.

   Or clear the quarantine flag from the terminal and launch normally:

   ```bash
   xattr -dr com.apple.quarantine "/Applications/JAW Quick Look.app"
   open "/Applications/JAW Quick Look.app"
   ```

   Launching registers the file type and the extension with Launch Services.
   You can quit the window afterward; the preview keeps working.

4. Select a `.jaw` file in Finder and press <kbd>Space</kbd>.

If the preview does not appear right away, nudge Launch Services and the
Quick Look daemon:

```bash
/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister \
  -f "/Applications/JAW Quick Look.app"
qlmanage -r && qlmanage -r cache
```

## Option B: Build from source

Needs Xcode 16 or later. See
[editors/macos/README.md](../../editors/macos/README.md) for the build and
signing steps; the install steps are the same as above from step 2, minus the
Gatekeeper prompt when you built it yourself.

## Uninstall

Delete `JAW Quick Look.app` from `/Applications`. Finder forgets the file type
and the preview the next time Launch Services rescans, or immediately with:

```bash
/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister \
  -u "/Applications/JAW Quick Look.app"
```
