---
name: jaw-review
description: Use when the user wants to lint or review a `.jaw` file for semantic issues the parser/LSP doesn't catch — undefined function calls, unused variables, style violations (em dashes, title case, bare function refs), decorator hygiene. Triggers on "review this .jaw", "lint my JAW", "check this JAW file before commit". For writing JAW, use `jaw-author`. For translating JAW, use `jaw-translate`.
allowed-tools: Read, Grep, Glob, Bash
---

# Reviewing a JAW file

The parser (`jaw-parse`) and LSP (`jaw-lsp`) already catch syntax errors and one warning ("did you mean `/func`?" for bare names matching a known function). **Do not duplicate that work.** This skill covers the semantic and style gaps.

## Step 1 — note the parser/LSP boundary

There is currently **no CLI frontend** for the parser. `jaw-cli`'s `export` subcommand iterates lines text-only and does not run `jaw-parse`. The only ways to get syntax diagnostics today are:

- Open the file in VS Code with the JAW extension installed (the LSP reports parse errors).
- Add a `jaw check` subcommand to `jaw-cli` (out of scope for this skill).

So: do not promise to run the parser. Mention to the user that for full syntax validation they should open the file in VS Code with the extension. Then proceed with the semantic and style checks below — these work on the raw file text and don't require a parsed AST.

## Step 2 — load the file and the spec

- Read the `.jaw` file fully.
- Read `jaw-grammar.md` and `docs/glossary.md` to ground the checks below.

## Step 3 — run the semantic checks

Report findings grouped by severity. For each, output `<file>:<line>: <message>`, matching the LSP source format (`jaw` as the source name).

### Errors

1. **Undefined function calls.** Any `/Foo[ ... ]` call where no `/Foo` definition exists in this file. (The LSP warns on the *inverse* — bare `Foo` matching a known def — but does not flag calls to undefined functions.)
   - Use Grep to find function definitions: `^/[A-Z]` patterns.
   - Use Grep to find function calls: `/[A-Z][A-Za-z0-9_]*\[` patterns.
   - Diff the call set against the definition set. Flag unmatched calls.
   - Exception: if the user indicates external functions are expected, mention but don't error.

### Warnings

2. **Unused variables.** A top-level `[X] — description` declared but never referenced in any step, conditional, loop, or function body. Inline assigns (`[X]: ...`) inside function bodies are scoped — only flag truly unreferenced top-level declarations.

3. **Bare function references in text.** A function name (matches a `/Foo` definition) appearing without the `/` prefix. The LSP already covers this — only mention it if the parser run in Step 1 surfaced such warnings.

### Style

4. **Em dash hygiene.** Every marker separator must be U+2014 `—`, not `-` or `--` or `–` (en dash). Grep for marker lines that use the wrong character: lines matching `^\s*\[[^\]]+\]\s*-+\s` (hyphen instead of em dash) or `^\s*/[A-Za-z][A-Za-z0-9_]*\s` followed by hyphen separators in args.

5. **Function name case.** Every `/<name>` must start with an uppercase letter and use title case. Flag `/lowercase`, `/snake_case`, `/camelCase`.

6. **Variable references in brackets.** Inside step text, names that look like declared variables but appear bare (e.g. `[1] — sort V` when `[V]` is declared). Lower confidence — only flag obvious cases where the bare name exactly matches a declared variable identifier.

### Decorator hygiene

7. **Unknown decorators.** Compile the set of decorators in use (`#name` or `#name:value`). Common decorators from `docs/examples/decorators.jaw` and the glossary: `#mutable`, `#pure`, `#error`, `#type`, `#complexity`. If the project has its own conventions doc, prefer that. Flag decorators that look like typos of known ones (e.g. `#mutabl`, `#pur`); do not flag unknown-but-plausible ones (the spec doesn't restrict the set).

## Step 4 — produce the report

Output format:

```
jaw-review: <file>

Errors (N):
  <file>:<line>: Undefined function call /Foo — no /Foo defined in this file
  ...

Warnings (N):
  <file>:<line>: Unused variable [X]
  ...

Style (N):
  <file>:<line>: Hyphen used instead of em dash —
  <file>:<line>: Function name /handle_pos should be title case (/HandlePos)
  ...

Decorators (N):
  <file>:<line>: Unknown decorator #mutabl — did you mean #mutable?
  ...

Clean checks: <list of categories that passed>
```

If everything is clean, say so explicitly and list which categories were checked. Don't invent issues to pad the output.

## What not to do

- Don't claim the parser was run when there's no CLI to invoke it. Be honest about the boundary (Step 1).
- Don't flag missing definitions for functions the user has indicated are external (e.g. stdlib-equivalent calls).
- Don't flag style issues in lines that are inside `[*]` or `[^]` comment continuations — those are free text.
- Don't auto-fix unless the user asks. Report first, fix on request.
