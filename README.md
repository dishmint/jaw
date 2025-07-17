# JAW

**JAW** (_just a word_) is a pseudocode language.

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
add [A]: an integer, [B]: an integer
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
[*] hey this code is pretty cool, or mayb I'll build a jungle gym tomorrow, whodafuq nose!!
```

#### Conditionals
To be frank, I don't like any of these. Something needs refinement. Ask what the problem is (and then what sub-problem needs solving in order to resolve it), that might clarify what's needed.
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

## Grammar
<!-- TODO: Grammar needs updating -->
```js
SOURCE        => REPEAT( TEXT | VARIABLE | INFIX | POSTFIX | PREFIX | FUNCTION | CODEBLOCK )
CODE          => REPEAT( TEXT )
INFIX         => [ ~ IDENTIFIER ~ ] — TEXT  
POSTFIX       => [ ~ IDENTIFIER   ] — TEXT  
PREFIX        => [   IDENTIFIER ~ ] — TEXT  
FUNCTION      => IDENTIFIER — REPEAT( VARIABLE ) : NEWLINE CODEBLOCK
VARIABLE      => [ IDENTIFIER ] — TEXT 
CODEBLOCK     => REPEAT( INDENT [ NUMBER ] — STEP NEWLINE )
STEP          => STATMENT | RETURN | COMMENT
STATEMENT     => [ NUMBER ] — CODE
RETURN        => [ > ] CODE
COMMENT       => [ ^ ] TEXT
/* [^] custom operators */
IDENTIFIER    => A-Z 0-9
TEXT          => PLAINTEXT
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
- [ ] TODO: Parallel Operations or sibling statements
