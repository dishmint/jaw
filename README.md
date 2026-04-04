# JAW

**JAW** (_just a word_) is a pseudocode and commenting language.

## Dictionary
### Variables
#### Global/Constant
```
[<variable-identifier>] — description

[V] — a 1D vector
```
#### Embedded
```
[<variable-identifier>]: description
[<variable-identifier>]: description = value

...
[Y]:a color = Red
...
```
### Functions
#### one-line header
```
/<function-identifier> [<argument-identifier>]: <argument-description>, ...
/add [A]: an integer, [B]: an integer
```

#### two-line header
```
/<function-identifier>
[<argument-identifier>]: <argument-description>, ...

/add
[A]: an integer, [B]: an integer
```

### Algorithm Steps
```
[<step-identifier>] — <abstract-step>

[1] — Do this thing
```

### Statements
#### Logging
```
[!] — <log-statement>
[!] I printy the fool!
```

#### Comments
```
[^] <code-comment>

[1] — 1 + 1
[^] add one to one to get two
```

```
[*] <general-comment>
[*] hey this code is pretty cool, or maybe I'll build a jungle gym tomorrow, whodafuq nose!!
```

#### Conditionals
To be frank, I don't like any of these. Something needs refinement. Ask what the problem is (and then what sub-problem needs solving in order to resolve it), that might clarify what's needed.

Conditionals are encounters. A condition is encountered that alters how the algorithm continues.
##### If/Else
```
[?] — <condition>
[?] — [A] > [B]
	[+] — Do this thing if true
	[-] — Do this thing if false
```
##### If/Else-If
```
[?] — [A] > [B]
	[+] — Do this thing if true
[?]	[-] — [B] < [C]
	[^] the 'else-if'
	[+] Do this thing
```
##### Ternary
```
[1] — [A] == [B] ? DoX : DoY

/DoX
...

/DoY
...
```
#### Returns
```
[>] <return-statement>

/get_in
[X]: a number
[>] get_out[ [X] ]

/get_out:
[>] 42
```

## Examples

```
[V1] — The first variable
[V2] — The second variable

/FUNC
[V1]: a 1D vector, [V2]: a 1D vector
	[1] — Zip[ [V1], [V2] ]
	[2] — For ( [A], [B] ) in [1]
		[1] — [A] + [B]
	[>] [2]

FUNC {1 2 3} {3 2 1} => {4 4 4}
[^] expected output
```

## TODOs
- [ ] TODO: Parallel Operations
- [ ] TODO: variable or statement decorators (annotating with additional information)

## Issues
One of the pain points in developing an explicit pseudocode language is determining how much of the PSC should be defined up front, and how much is at the user's discretion. JAW is flexible enough that plain text can be used in place of extact syntax. Meaning, one could write in natural language what a certain step should perform, or which function shouldbe called with what arguments.
