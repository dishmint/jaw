; Later patterns take precedence (Zed).

; Comments: [^] code comment, [*] general comment
(comment) @comment

; Logs: [•] marker, message as a string, `Title:` before continuation lines
(log_marker) @keyword
(log (text) @string)
(log (continuation (text) @string))
((log (text) @emphasis . (continuation))
  (#match? @emphasis ":\\s*$"))

; Important notes: [!] marker and message share the marker's colour
(note_marker) @keyword
(note (text) @keyword)
(note (continuation (text) @keyword))
((note (text) @emphasis.strong . (continuation))
  (#match? @emphasis.strong ":\\s*$"))

; Control markers
[
  (loop_marker)
  (parallel_marker)
  (true_marker)
  (false_marker)
  (return_marker)
] @keyword

(loop "in" @keyword)

; Steps [1] and step references
(step_ref) @number
(number) @number

; Descriptions after `[X]:` (inner refs keep their own colours below)
(description) @string

; Variables [V]
(variable_ref) @variable

; Functions /Name — a line starting with one defines it
(function_ref) @function
(line . (function_ref) @function.definition)

; Decorators #name / #name:value
(decorator "#" @attribute)
(decorator name: (identifier) @attribute)
(decorator value: (value) @string)

(operator) @operator
((operator) @keyword
  (#match? @keyword "^[?|]$"))
(em_dash) @punctuation.delimiter
[":" "," ";" "."] @punctuation.delimiter
["[" "]" "(" ")"] @punctuation.bracket
