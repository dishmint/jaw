; Helix: the first pattern that matches a node wins, so specific patterns come
; before general ones (the reverse of the Zed copy).

; Titles: `[•] — Title:` / `[!] — Title:` followed by continuation lines
((log (text) @markup.italic . (continuation))
  (#match? @markup.italic ":\\s*$"))
((note (text) @markup.bold . (continuation))
  (#match? @markup.bold ":\\s*$"))

; Logs: [•] marker, message as a string
(log_marker) @keyword
(log (text) @string)
(log (continuation (text) @string))

; Important notes: [!] marker and message share the marker's colour
(note_marker) @keyword
(note (text) @keyword)
(note (continuation (text) @keyword))

; Comments: [^] code comment, [*] general comment
(comment) @comment

; Control markers
[
  (loop_marker)
  (parallel_marker)
  (true_marker)
  (false_marker)
  (return_marker)
] @keyword.control

(loop "in" @keyword.control)

; Steps [1] and step references
(step_ref) @constant.numeric
(number) @constant.numeric

; Variables [V]
(variable_ref) @variable

; Descriptions after `[X]:`
(description) @string

; Functions /Name
(function_ref) @function

; Decorators #name / #name:value
(decorator "#" @attribute)
(decorator name: (identifier) @attribute)
(decorator value: (value) @string)

((operator) @keyword.control.conditional
  (#match? @keyword.control.conditional "^[?|]$"))
(operator) @operator

(em_dash) @punctuation.delimiter
[":" "," ";" "."] @punctuation.delimiter
["[" "]" "(" ")"] @punctuation.bracket
