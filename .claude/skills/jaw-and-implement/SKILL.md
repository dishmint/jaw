---
name: jaw-and-implement
description: Use when the user wants BOTH JAW pseudocode AND a working implementation together — e.g. "write JAW and Python for X", "spec plus code", "pseudocode and implement it in Rust". Output is a single source file with JAW lines as comments above the real code (mirrors `jaw export` shape). For JAW alone, use `jaw-author`. For converting an existing `.jaw` file to code, use `jaw-translate`.
allowed-tools: Read, Write, Edit, Glob, Grep
---

# JAW + implementation in one file

JAW is a **companion** to code, not a precursor. The output is a single file in the target language where JAW lines appear as comments above the code they describe. This mirrors what `jaw-cli export` produces — except here you write both the JAW and the code in one pass, so they stay in sync.

**Do not** emit two separate fenced blocks (JAW first, code second). They belong adjacent.

## Step 1 — confirm the target language

If the user didn't specify, ask. Supported comment styles (matching `jaw-cli`'s language table — see `jaw-cli/src/lang.rs`):

| Language | Line comment | Block comment |
|----------|--------------|---------------|
| Python, Ruby, Bash | `#` | (none — use per-line) |
| Rust, JS, TS, Go, C, C++ | `//` | `/* */` (for multi-line `[*]`/`[^]`) |
| Lua | `--` | `--[[ ]]` |
| SQL | `--` | (none) |

## Step 2 — design the JAW

Before writing anything, draft the JAW algorithm mentally. Follow the rules from `.claude/skills/jaw-author/SKILL.md`:

- Em dash `—` (U+2014), `/Title` for function refs, `[V]` for variable refs, `#name` for decorators.
- Pick the right markers: `[N] —` step, `[~] —` loop, `[+]`/`[-]` complex conditional branches, `[!] —` log, `[>]` return, `[V] —` top-level variable, `[V]:` inline assign.
- Function names are title case (`/BinarySearch`, not `/binary_search`).
- **Conditionals with `?` require function calls**, per the grammar: `CODE ? FUNCTION_CALL | FUNCTION_CALL`. Don't write `[X] ? returnY | doZ` — either use real function refs (`? /Found | /Continue`) and define them, or describe the conditional in plain step text (`return [Y] if [X]`), or use the complex `[+]`/`[-]` form.

If the JAW is non-trivial, read `docs/examples/example-func.jaw` to calibrate.

## Step 3 — map JAW to code idioms

Same as `.claude/skills/jaw-translate/SKILL.md` Mode 1:

- `[~] — [X] in [Coll]` → for-each loop
- `[~] — condition` → while loop
- `[+] —` / `[-] —` complex conditional → if/else
- `[N] — [A] > [B] ? /DoX | /DoY` chained conditional → if/else **dispatching to the named functions**
- `[!] —` → log/print
- `[>] expr` → return
- `[R] << x` → append `x` to collection
- `[V]@[P]` → indexed access
- Inline assigns with `= value` → parameter defaults

## Step 4 — emit a single fenced block

Wrap the output in one fenced code block for the target language. Place JAW lines as comments **above** the code they describe.

### Format conventions

- **Top-level variable declarations** (`[V] — description`) → comment above the function signature, or above the relevant initialization.
- **Function header** (`/Name [A]: ..., [B]: ...`) → comment above the function definition.
- **Steps** (`[N] — ...`) → comment above the line(s) implementing that step. If a step is one line of code, the comment goes directly above. If it's several, the comment goes above the first.
- **Inline assigns** (`[X]: desc = val`) → comment above the corresponding variable assignment.
- **Loops, conditionals** (`[~] —`, `[+]`, `[-]`) → comment above the `while`/`for`/`if`/`else` line.
- **Logs and returns** (`[!] —`, `[>]`) → comment above the print/return line.
- **`[^]` code comments** and **`[*]` general comments** → keep as-is in comment form; multi-line ones can use block comment syntax in C-style languages.

### Example shape (Python target)

```python
# /BinarySearch [V]: sorted list, [T]: target
def binary_search(v, t):
    # [L]: low = 0
    # [H]: high = len([V]) - 1
    low, high = 0, len(v) - 1

    # [~] — [L] <= [H]
    while low <= high:
        # [1] — [M]: midpoint
        mid = (low + high) // 2

        # [2] — return [M] if [V]@[M] == [T]
        if v[mid] == t:
            return mid

        # [3] — narrow [L]..[H] toward [T]
        if v[mid] < t:
            low = mid + 1
        else:
            high = mid - 1

    # [>] -1
    return -1
```

Note: steps `[2]` and `[3]` use plain step text rather than the `?` chained-conditional form, because the branches are inline operations (return, assignment) — not function calls. Using `?` would require defining `/Found`, `/SearchRight`, `/SearchLeft` and dispatching to them.

### Example shape (Rust target — block comment for multi-line)

```rust
// /BinarySearch [V]: sorted slice, [T]: target
fn binary_search(v: &[i32], t: i32) -> Option<usize> {
    // [L]: low = 0, [H]: high = len([V]) - 1
    let (mut low, mut high) = (0_isize, v.len() as isize - 1);

    // [~] — [L] <= [H]
    while low <= high {
        // [1] — [M]: midpoint
        let mid = ((low + high) / 2) as usize;

        // [2] — return [M] if [V]@[M] == [T]
        if v[mid] == t {
            return Some(mid);
        }

        // [3] — narrow [L]..[H] toward [T]
        if v[mid] < t {
            low = mid as isize + 1;
        } else {
            high = mid as isize - 1;
        }
    }

    // [>] None
    None
}
```

## Step 5 — self-check before returning

1. **JAW rules:** em dashes U+2014, slashed function refs, title-case names, bracketed variables.
2. **Grammar:** any `?` chained conditional dispatches to actual function calls (which exist or are clearly external) — no dangling `/Foo` refs.
3. **Code correctness:** the code actually compiles/runs (mentally trace the golden path).
4. **Sync:** every JAW step has a matching code section; every meaningful code section has a JAW comment above it. No orphans on either side.
5. **Comment syntax:** matches the target language. Use block comments (`/* */`) for multi-line continuations in C-style languages.

## Multiple target languages

If the user asks for the same algorithm in multiple languages, emit one fenced block per language, each with its own interleaved JAW comments. Keep the JAW content identical across blocks (only the comment syntax changes).

## When to suggest `jaw-cli export` instead

If the user already has a `.jaw` file and just wants it converted to a code skeleton (comments only, no implementation), point them at `jaw-cli`:

```bash
~/.cargo/bin/cargo run -p jaw-cli -- export --lang <LANG> <FILE.jaw>
```

That's deterministic and handles all the comment-syntax mapping. This skill is for the case where they want *both* the comments AND a real implementation generated together.
