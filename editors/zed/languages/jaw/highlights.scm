; Later patterns take precedence (Zed).

; Comments: [^] code comment, [*] general comment
(comment) @comment
(comment (variable_ref) @variable)

; Logs: [•] marker, message as a string, `Title:` before continuation lines
(log_marker) @keyword
(log (text) @string)
(log (continuation) @string)
((log (text) @emphasis . (continuation))
  (#match? @emphasis ":\\s*$"))

; Important notes: [!] marker and message share the marker's colour
(note_marker) @keyword
(note (text) @keyword)
(note (continuation) @keyword)
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
(em_dash) @punctuation.delimiter
[":" "," ";" "."] @punctuation.delimiter
["[" "]" "(" ")"] @punctuation.bracket
