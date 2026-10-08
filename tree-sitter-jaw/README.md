# tree-sitter-jaw

Tree-sitter grammar for JAW. Used by the [Zed extension](../editors/zed/), and intended for Helix (#16) and Neovim (#15).

The grammar is line-oriented and exists for highlighting: each line is a `comment`, `log`, `note`, `loop` header or generic `line` of tokens. It does not model function bodies or nesting — `jaw-parse` remains the source of truth for structure. Lines after a comment, log or note become `continuation` nodes of it. The external scanner (`src/scanner.c`) decides this, using the same rules as the VS Code TextMate grammar:

- `[^]` / `[*]` comment: continues until the next non-blank line starts with a JAW construct (`[N]`, a marker, `[ID]` followed by `—` or `:`, or `/`).
- `[•]` log / `[!]` note: continues while the next non-blank line is indented deeper than the marker.

## Develop

```sh
npm i -g tree-sitter-cli   # or: npx tree-sitter-cli
tree-sitter generate --abi 14
tree-sitter test
tree-sitter parse ../samples/full.jaw
```

Commit the regenerated `src/` with any `grammar.js` change.

`queries/highlights.scm` is copied to `editors/zed/languages/jaw/highlights.scm`; keep them identical. Patterns are ordered general to specific because Zed gives later patterns precedence.
