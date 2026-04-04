# JAW

**JAW** (_just a word_) is a pseudocode and commenting language.

## Syntax

### Variables

#### Global/Constant
```
[<identifier>] — description

[V] — a 1D vector
```

#### Inline Assignment
```
[<identifier>]: description
[<identifier>]: description = value

[L]: length of [V]
[R]: result
[L]: length of [V] = 0
[Y]: a color = Red
```

### Functions

#### One-line header
```
/<function-identifier> [<arg>]: <description>, ...

/add [A]: an integer, [B]: an integer
```

#### Two-line header
```
/<function-identifier>
[<arg>]: <description>, ...

/add
[A]: an integer, [B]: an integer
```

### Algorithm Steps
```
[<step-number>] — <action>

[1] — Do this thing
```

### Comments

#### Code Comments
Placed under a step to describe it. Supports multi-line: the marker starts the comment, plain-text lines continue until the next JAW construct.
```
[1] — 1 + 1
[^] add one to one to get two

[1] — sort [V]
[^] this sorts the vector
using a quicksort variant
because the data is mostly sorted
[2] — next step
```

#### General Comments
For notes not tied to a specific step. Also supports multi-line continuation.
```
[*] hey this code is pretty cool
[*] here's a longer thought
that spans multiple lines
and keeps going until the next construct
```

### Logging
```
[!] — <log-statement>

[!] — I printy the fool!
```

### Returns
```
[>] <return-value>

/get_in
[X]: a number
[>] get_out[ [X] ]

/get_out
[>] 42
```

### Conditionals

#### Simple (one-liner)
```
[1] — [A] > [B] ? DoX | DoY
```

#### Chained (else-if)
```
[1] — [A] > [B] ? DoX | [A] == [B] ? DoY | DoZ
[^] if A>B do X, else if A==B do Y, else do Z
```

#### Complex (multi-step branches)
When branches require multiple steps, use `[+]` and `[-]` blocks:
```
[1] — [A] > [B] ?
	[+] — DoX
	[-] — DoY
```

### Loops

`[~]` is the loop marker. The text after it describes what governs the repetition — a condition (while-style) or an iteration expression (for-each-style).

#### While
```
[1] — [P]: position = 0
[~] — [P] < [L]
	[1] — do something
	[2] — [P] += 1
```

#### For-each
```
[~] — [X] in [V]
	[1] — do something with [X]
```

#### Destructured iteration
```
[1] — Zip[ [V1], [V2] ]
[~] — ([A], [B]) in [1]
	[1] — [A] + [B]
```

### Parallel Operations

`[&]` marks steps that execute concurrently. Sibling steps nested under `[&]` run together, and the algorithm continues after all complete.

```
[&]
	[1] — Fetch data from API
	[2] — Load cache from disk
[2] — Process results
[^] step 2 waits for [&] to complete
```

### Decorators

`#name` or `#name:value` annotations attach metadata to variables, steps, or functions.

```
[V] — a vector #mutable #type:list
[1] — sort [V] #complexity:O(nlogn)

/add #pure
[A]: an integer, [B]: an integer
[>] [A] + [B]
```

Function-level decorators go on the `/name` line. Decorators are flexible — the user decides what to annotate.

### Array Access

`@` is used for accessing elements in a collection by index:
```
[V]@[P]
[^] element at position P in V
```

### Operators

Standard math and programming notation: `+`, `-`, `*`, `/`, `==`, `>`, `<`, `>=`, `<=`, `!=`, `<<` (append), etc.

Custom operations are just functions:
```
/Add [A]: an integer, [B]: an integer
[>] [A] + [B]
```

## Examples

```
[V1] — The first variable
[V2] — The second variable

/FUNC
[V1]: a 1D vector, [V2]: a 1D vector
	[1] — Zip[ [V1], [V2] ]
	[R]: result
	[~] — ([A], [B]) in [1]
		[1] — [R] << [A] + [B]
		[^] append sum to R
	[>] [R]

FUNC {1 2 3} {3 2 1} => {4 4 4}
[^] expected output
```

```
[V] — a vector

/obverse
[V]: a list
	[1] — [L]: length of [V]
	[^] gives us an upper bound
	[2] — [P]: tracking position = 0
	[^] allows us to move along [V]
	[~] — [P] < [L]
		[!] — `[V][ [P] ]` @ v[`[P]`]
		[^] the [!] above means log or print
		[1] — [P] += 1
		[^] advance position for the next iteration
```

```
[V] — a vector

/obverse_skipnegative
[V]: a list
	[1] — [L]: length of [V]
	[^] gives us an upper bound
	[2] — [P]: tracking position = 0
	[^] allows us to move along [V]
	[~] — [P] < [L]
		[1] — [V]@[P] > 0 ? LogState | Pass
		[^] log if positive, otherwise continue
		[2] — [P] += 1
		[^] advance tracking position

/LogState
	[!] — `[V][ [P] ]` @ v[`[P]`]
```

## Issues

One of the pain points in developing an explicit pseudocode language is determining how much of the syntax should be defined up front, and how much is at the user's discretion. JAW is flexible enough that plain text can be used in place of exact syntax. Meaning, one could write in natural language what a certain step should perform, or which function should be called with what arguments.
