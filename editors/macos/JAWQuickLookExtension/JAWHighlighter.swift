import Foundation

// A small, dependency-free JAW highlighter. It mirrors the constructs the
// TextMate grammar (editors/vscode/syntaxes/jaw.tmLanguage.json) recognizes,
// but as a line-oriented tokenizer that emits HTML spans. It is intentionally
// approximate — a Quick Look preview only needs to read well, not to match the
// parser exactly — and it never throws: anything it can't classify falls through
// as plain, escaped text.
enum JAWHighlighter {
    static func html(for source: String) -> String {
        let lines = source.components(separatedBy: "\n")
        let body = lines.map(renderLine).joined(separator: "\n")
        return page(body: body)
    }

    // MARK: - Line rendering

    private static let markerSymbols: Set<String> =
        ["^", "*", "•", "!", ">", "~", "&", "+", "-"]

    private static func renderLine(_ line: String) -> String {
        let (indent, rest) = splitLeadingWhitespace(line)
        let lineClass = lineLevelClass(for: rest)
        let inner = escape(indent) + tokenize(rest)
        return "<span class=\"ln\(lineClass.map { " " + $0 } ?? "")\">\(inner)</span>"
    }

    // Classify the whole line by its leading marker so notes/logs/comments can
    // carry the spec's emphasis (notes are red + bold, comments muted, etc.).
    private static func lineLevelClass(for rest: String) -> String? {
        if rest.hasPrefix("[!]") { return "note" }
        if rest.hasPrefix("[•]") { return "log" }
        if rest.hasPrefix("[*]") || rest.hasPrefix("[^]") { return "comment" }
        return nil
    }

    // MARK: - Tokenizing

    // One pass, non-overlapping. Order inside the alternation matters only where
    // patterns could both start at the same index; brackets and function refs
    // are anchored to distinct lead chars so they don't collide.
    private static let tokenRegex: NSRegularExpression = {
        let pattern = [
            "(/[A-Za-z][A-Za-z0-9_]*)",                       // 1 function ref
            "(\\[[^\\]\\n]*\\])",                              // 2 bracketed token
            "(#[A-Za-z][A-Za-z0-9_]*(?::[^\\s]+)?)",          // 3 decorator
            "(—)",                                            // 4 em dash separator
            "(\\+=|\\*\\*|==|!=|>=|<=|<<|[-+*/<>=?|@:])",      // 5 operator / access
            "(\\b\\d+(?:\\.\\d+)?\\b)"                         // 6 number
        ].joined(separator: "|")
        // swiftlint:disable:next force_try
        return try! NSRegularExpression(pattern: pattern)
    }()

    private static func tokenize(_ text: String) -> String {
        guard !text.isEmpty else { return "" }
        let ns = text as NSString
        let full = NSRange(location: 0, length: ns.length)
        var out = ""
        var cursor = 0

        tokenRegex.enumerateMatches(in: text, range: full) { match, _, _ in
            guard let match = match else { return }
            let r = match.range
            if r.location > cursor {
                let gap = ns.substring(with: NSRange(location: cursor, length: r.location - cursor))
                out += escape(gap)
            }
            let token = ns.substring(with: r)
            out += span(for: token, match: match)
            cursor = r.location + r.length
        }

        if cursor < ns.length {
            out += escape(ns.substring(from: cursor))
        }
        return out
    }

    private static func span(for token: String, match: NSTextCheckingResult) -> String {
        // Which alternation group fired tells us the kind without re-matching.
        if match.range(at: 1).location != NSNotFound {
            return wrap(token, "fn")
        }
        if match.range(at: 2).location != NSNotFound {
            return bracketSpan(token)
        }
        if match.range(at: 3).location != NSNotFound {
            return wrap(token, "deco")
        }
        if match.range(at: 4).location != NSNotFound {
            return wrap(token, "sep")
        }
        if match.range(at: 5).location != NSNotFound {
            return wrap(token, "op")
        }
        if match.range(at: 6).location != NSNotFound {
            return wrap(token, "num")
        }
        return escape(token)
    }

    // `[42]` is a step, `[^]`/`[~]`/… are markers, everything else (`[V]`, `[ID]`)
    // is a variable reference.
    private static func bracketSpan(_ token: String) -> String {
        let inner = String(token.dropFirst().dropLast())
        if !inner.isEmpty && inner.allSatisfy({ $0.isNumber }) {
            return wrap(token, "step")
        }
        if markerSymbols.contains(inner) {
            return wrap(token, "marker")
        }
        return wrap(token, "var")
    }

    // MARK: - HTML helpers

    private static func wrap(_ text: String, _ cls: String) -> String {
        "<span class=\"\(cls)\">\(escape(text))</span>"
    }

    private static func splitLeadingWhitespace(_ line: String) -> (String, String) {
        guard let idx = line.firstIndex(where: { !$0.isWhitespace || $0 == "\n" }) else {
            return (line, "")
        }
        return (String(line[..<idx]), String(line[idx...]))
    }

    private static func escape(_ s: String) -> String {
        var r = ""
        r.reserveCapacity(s.count)
        for ch in s {
            switch ch {
            case "&": r += "&amp;"
            case "<": r += "&lt;"
            case ">": r += "&gt;"
            default: r.append(ch)
            }
        }
        return r
    }

    // MARK: - Page shell

    private static func page(body: String) -> String {
        """
        <!DOCTYPE html>
        <html>
        <head>
        <meta charset="utf-8">
        <meta name="viewport" content="width=device-width, initial-scale=1">
        <style>
        \(css)
        </style>
        </head>
        <body>
        <pre class="jaw">\(body)</pre>
        </body>
        </html>
        """
    }

    // Palette tracks the VS Code extension where it can (the [•] log amber comes
    // straight from extension.ts). Light and dark variants via prefers-color-scheme.
    private static let css = """
    :root {
      --bg: #ffffff; --fg: #1f2328;
      --marker: #6f42c1; --step: #0969da; --var: #0a7ea4;
      --fn: #8250df; --deco: #953800; --sep: #6e7781;
      --op: #cf222e; --num: #0550ae; --log: #b45309;
      --comment: #6e7781; --note: #cf222e;
    }
    @media (prefers-color-scheme: dark) {
      :root {
        --bg: #1e1e1e; --fg: #d4d4d4;
        --marker: #c586c0; --step: #569cd6; --var: #4ec9b0;
        --fn: #dcdcaa; --deco: #ce9178; --sep: #808080;
        --op: #d16969; --num: #b5cea8; --log: #d7ba7d;
        --comment: #6a9955; --note: #f14c4c;
      }
    }
    html, body { margin: 0; padding: 0; background: var(--bg); }
    pre.jaw {
      margin: 0;
      padding: 16px 20px;
      background: var(--bg);
      color: var(--fg);
      font-family: ui-monospace, "SF Mono", Menlo, Monaco, "Cascadia Code", monospace;
      font-size: 13px;
      line-height: 1.5;
      white-space: pre;
      tab-size: 4;
      -moz-tab-size: 4;
    }
    .ln { display: block; min-height: 1.5em; }
    .marker { color: var(--marker); font-weight: 600; }
    .step   { color: var(--step); font-weight: 600; }
    .var    { color: var(--var); font-weight: 600; }
    .fn     { color: var(--fn); }
    .deco   { color: var(--deco); }
    .sep    { color: var(--sep); }
    .op     { color: var(--op); }
    .num    { color: var(--num); }
    .ln.note    { color: var(--note); font-weight: 700; }
    .ln.note .marker { color: var(--note); }
    .ln.log .marker  { color: var(--log); }
    .ln.comment { color: var(--comment); font-style: italic; }
    .ln.comment .var, .ln.comment .marker { color: inherit; }
    """
}
