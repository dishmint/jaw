# tree-sitter-jaw

Tree-sitter grammar for JAW. Used by the [Zed extension](../editors/zed/), and intended for Helix (#16) and Neovim (#15).

The grammar is line-oriented and exists for highlighting: each line is a `comment`, `log`, `note`, `loop` header or generic `line` of tokens. It does not model function bodies or nesting — `jaw-parse` remains the source of truth for structure. Plain-text lines after a comment, log or note become `continuation` nodes of it.

## Develop

```sh
npm i -g tree-sitter-cli   # or: npx tree-sitter-cli
tree-sitter generate --abi 14
tree-sitter test
tree-sitter parse ../samples/full.jaw
```

Commit the regenerated `src/` with any `grammar.js` change.

`queries/highlights.scm` is copied to `editors/zed/languages/jaw/highlights.scm`; keep them identical. Patterns are ordered general to specific because Zed gives later patterns precedence.
