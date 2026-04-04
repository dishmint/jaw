# JAW Glossary

## Markers

| Marker | Name | Description |
|--------|------|-------------|
| `[<id>]` | Variable | A named reference, enclosed in brackets |
| `[<id>] —` | Variable Declaration | Defines a global variable with a description |
| `[<id>]:` | Inline Assignment | Assigns a description (and optional value) to a variable |
| `/<name>` | Function | Defines or references a function |
| `[<n>] —` | Step | A numbered algorithm step |
| `[^]` | Code Comment | A comment tied to the preceding step |
| `[*]` | General Comment | A standalone comment, not tied to a step |
| `[!] —` | Log | A log/print statement |
| `[>]` | Return | Returns a value from a function |
| `[~] —` | Loop | Repeats steps governed by a condition or iterator |
| `[&]` | Parallel | Marks steps that execute concurrently |
| `[+] —` | True Branch | The true path in a complex conditional |
| `[-] —` | False Branch | The false path in a complex conditional |
| `?` | Conditional | Begins a conditional branch |
| `\|` | Else | Separates conditional branches |
| `@` | Access | Accesses an element by index: `[V]@[P]` |
| `#` | Decorator | Attaches metadata: `#name` or `#name:value` |
| `<<` | Append | Appends a value to a collection |
| `—` | Em Dash | Separates a marker from its content |

## Terms

| Term | Definition |
|------|------------|
| **Step** | A single action in an algorithm, numbered sequentially |
| **Code Block** | An indented group of steps belonging to a function, loop, or parallel block |
| **Decorator** | Metadata annotation on a variable, step, or function (e.g., `#mutable`, `#error`) |
| **Inline Assignment** | Declaring a variable within a code block, with an optional value |
| **Continuation** | Lines following a comment or log that extend its content until the next JAW construct |
| **Chained Conditional** | Multiple conditions linked with `?` and `\|` on a single line |
| **Complex Conditional** | A conditional with `[+]`/`[-]` blocks for multi-step branches |
| **Destructured Iteration** | A for-each loop that unpacks multiple variables: `([A], [B]) in [1]` |
