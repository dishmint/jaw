# Grammar
wip grammar for JAW. Once the v1 spec is locked in I'll work on the grammar / lsp etc.
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