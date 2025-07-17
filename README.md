# JAW

**JAW** (_just a word_) is a pseudocode language.

## Grammar

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

FUNC [V1] [V2] :
	[1] — Do X to [V1]
	[2] — Do Y to [V2]
	[3] — [1] + [2]
	[>] [3]
```

```
[V1] — The first variable
[V2] — The second variable

FUNC [V1] [V2] :
	[1] — Do X to [V1]
	[2] — Do Y to [V2]
	[>] [1] + [2]
```

```
[V1] — The first variable
[V2] — The second variable

FUNC [V1] [V2] :
	[1] — X[ V1 ]
	[2] — Y[ V2 ]
	[>] [1] + [2]
```

## ROADMAP
- [ ] Conditionals
- [ ] Loops
- [ ] Parallel Operations
