---
name: jaw-author
description: Use when the user wants to write JAW (Just A Word) pseudocode from a description, convert an algorithm or function into JAW, or asks what a JAW marker means. Triggers on `.jaw` files, "write JAW for X", "explain `[~]`", "what does this decorator do". For converting JAW into a real programming language, use `jaw-translate` instead. For linting an existing `.jaw` file, use `jaw-review`.
allowed-tools: Read, Write, Edit, Glob, Grep
---

# Authoring JAW pseudocode

JAW is a pseudocode/commenting language. Conventions are precise — if you guess from training data you will get details wrong. **Always load the spec before writing.**

## Step 1 — load the spec

Read these three files before producing JAW:

- `jaw-grammar.md` — formal EBNF-style grammar (~60 lines)
- `docs/glossary.md` — marker and term reference
- `CLAUDE.md` — project conventions

If the user only asked a definitional question ("what does `[~]` mean"), `docs/glossary.md` is enough.

## Step 2 — apply the critical rules

These rules are easy to get wrong and the parser/LSP will not all flag them:

| Rule | Right | Wrong |
|------|-------|-------|
| Em dash separator | `—` (U+2014) | `-`, `--`, `–` |
| Function refs | `/Add`, `/Process` | `Add`, `add` |
| Function names | Title case `/HandlePositive` | `/handle_positive`, `/handlePositive` |
| Variable refs | `[V]`, `[Result]` | `V`, `<V>` |
| Decorators | `#mutable`, `#type:list` | `@mutable`, `mutable:` |
| Array access | `[V]@[P]` | `[V][P]`, `[V].P` |

## Step 3 — pick the right marker

| Marker | When |
|--------|------|
| `[N] —` | A numbered algorithm step inside a function/loop/parallel block |
| `[~] —` | Loop. Body is an indented block. Iteration form: `[A] in [Coll]` or `([A], [B]) in [Pairs]` |
| `[&]` | Parallel block. Body is indented. |
| `[+] —` / `[-] —` | True/false branches of a complex (multi-step) conditional |
| `[•] —` | Log / print |
| `[!] —` | Important note (rendered red and bold) |
| `[>]` | Return |
| `[^]` | Code comment tied to the preceding step |
| `[*]` | Standalone general comment |
| `[V] —` | Variable declaration (at top level) |
| `[V]:` | Inline assignment (inside a code block); optional `= value` |

Conditionals come in two shapes:
- **Chained on one line:** `[1] — [A] > [B] ? /DoX | [A] == [B] ? /DoY | /DoZ`
- **Complex with multi-step branches:**
  ```
  [1] — [A] > [B] ?
      [+] — /DoX
      [-] — /DoY
  ```

## Step 4 — reference real examples

When in doubt, read the matching example file rather than improvising:

| Topic | File |
|-------|------|
| Function declarations | `docs/examples/functions.jaw` |
| Variables and inline assignments | `docs/examples/variables.jaw` |
| Conditionals (chained + complex) | `docs/examples/conditionals.jaw` |
| Loops and destructured iteration | `docs/examples/loops.jaw` |
| Decorators | `docs/examples/decorators.jaw` |
| Logging | `docs/examples/logging.jaw` |
| Notes (important) | `docs/examples/notes.jaw` |
| Returns | `docs/examples/returns.jaw` |
| End-to-end algorithm (Zip + sum) | `docs/examples/example-func.jaw` |
| Larger reference | `samples/full.jaw` |

## Step 5 — self-check before returning

Scan your output for these failure modes:

1. **Em dashes:** every `[X] —` and `[N] —` uses U+2014, not `-` or `--`.
2. **Function name case:** every `/Name` is title case.
3. **Function refs are slashed:** function names appearing in step text (e.g. inside `[N] — result = Foo[...]`) must be `/Foo`, not bare `Foo`. The LSP will warn about this; you should not produce it.
4. **Variables in brackets:** every variable reference uses `[V]`, not bare `V`.
5. **Indentation:** code blocks inside functions/loops are indented one level.

## Output conventions

- For a single function or snippet, return JAW in a fenced block:
  ```jaw
  /MyFunc
  [V]: input
      [1] — ...
      [>] [V]
  ```
- For multi-file work, write to `.jaw` files with the Write tool.
- Don't surround JAW with explanatory prose unless the user asked for an explanation — JAW *is* the explanation.
