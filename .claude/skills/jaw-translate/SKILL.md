---
name: jaw-translate
description: Use when the user wants to convert between JAW pseudocode and real source code. Three modes — (1) implement a `.jaw` file as runnable code in Python/Rust/TS/etc., (2) embed a `.jaw` file as comments inside a source file in any supported language, (3) summarize an existing function as JAW pseudocode. Triggers on "implement this JAW in <lang>", "translate this to JAW", "embed this as comments". For writing JAW from scratch, use `jaw-author`. For linting JAW, use `jaw-review`.
allowed-tools: Read, Write, Edit, Bash, Glob, Grep
---

# Translating between JAW and real code

Pick the mode based on what the user asked for. The three modes use different tools.

## Mode 1 — JAW → runnable code (LLM task)

The user wants a working implementation in a target language, not comments.

1. Read the `.jaw` file with the Read tool.
2. Infer types and semantics:
   - Variable descriptions (`[V] — a 1D vector`) hint at types.
   - Decorators carry intent: `#mutable` (assignable), `#pure` (no side effects), `#type:list`, `#error`, `#complexity:O(n)`.
   - Inline assigns with defaults (`[N]: count = 0`) become parameter defaults.
   - Function args `[A]: an integer, [B]: an integer` map to typed parameters.
3. Translate construct by construct. Map JAW shapes to language idioms — not literal one-to-one. Examples:
   - `[~] — [X] in [Coll]` → `for x in coll:` (or the language equivalent)
   - `[~] — ([A], [B]) in [Pairs]` → destructured for-each
   - `[+] — ... / [-] — ...` → if / else
   - `[!] — message` → log/print (use the project's logger if visible in context)
   - `[>] expr` → return
   - `[N] — [R] << x` → append `x` to `[R]` (collection mutation)
4. **Reference worked example:** `docs/examples/example-func.jaw` is a Zip + sum algorithm. Use it to calibrate how decorators and inline assigns translate.
5. If the JAW references `/Foo` but no `/Foo` is defined in the file, ask the user whether `Foo` is an external function or whether they want it stubbed.

## Mode 2 — JAW → comments in a source file (deterministic)

The user wants their JAW embedded as comments — they're not asking for a real implementation, just a commented scaffold.

Use the `jaw-cli` binary. **Do not reimplement comment formatting** — the CLI handles per-language syntax, multi-line block comments (`/* ... */` for C-style languages), and indentation preservation.

```bash
~/.cargo/bin/cargo run -p jaw-cli -- export --lang <LANG> <INPUT.jaw> --output <OUTPUT>
```

Supported languages (with aliases): `python`/`py`, `ruby`/`rb`, `bash`/`sh`, `rust`/`rs`, `javascript`/`js`, `typescript`/`ts`, `go`, `c`, `cpp`/`c++`, `lua`, `sql`. Source: `jaw-cli/src/lang.rs`.

If the user wants the result printed to stdout, omit `--output`.

If `jaw-cli` is not built, build first: `~/.cargo/bin/cargo build -p jaw-cli`.

## Mode 3 — code → JAW pseudocode (LLM task)

The user wants an existing function summarized as JAW comments/pseudocode.

1. Read the source file and identify the function to translate.
2. Load the JAW conventions first — read `jaw-grammar.md` and `docs/glossary.md`. **Critical rules** (or the LSP will warn / output will be invalid):
   - Em dash `—` is U+2014, not `-` or `--`.
   - Function references use `/Name` with title case (`/HandlePositive`).
   - Variable references are bracket-enclosed: `[V]`, `[Result]`.
   - Decorators use `#name` or `#name:value`.
   - Array access is `[V]@[P]`, not `[V][P]`.
3. Produce JAW. Map back from idioms:
   - `for x in coll:` → `[~] — [X] in [Coll]`
   - if/else with multi-line branches → `[+] —` / `[-] —`
   - return → `[>]`
   - print/log → `[!] —`
   - typed parameters → inline assigns: `/Func [A]: an integer, [B]: a list`
4. Pick the right granularity. Each numbered `[N] —` step should correspond to one meaningful action, not every line of code. JAW is for the algorithm, not a literal transliteration.
5. Self-check using the same checklist as `jaw-author` — em dashes, slashed function refs, title-case names, bracketed variables.

## Picking between modes

If the user's request is ambiguous, ask. Mode 1 ("runnable code") and Mode 2 ("comments in a source file") look similar but produce very different output — get this right before generating.
