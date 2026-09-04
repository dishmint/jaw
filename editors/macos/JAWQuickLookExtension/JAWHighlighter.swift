import AppKit

// A small, dependency-free JAW highlighter. It mirrors the constructs the
// TextMate grammar (editors/vscode/syntaxes/jaw.tmLanguage.json) recognizes,
// but as a line-oriented tokenizer that emits an attributed string for an
// NSTextView. It is intentionally approximate — a Quick Look preview only
// needs to read well, not to match the parser exactly — and it never throws:
// anything it can't classify falls through as plain text.
//
// It used to emit HTML for a WKWebView. WebKit's content process does not
// survive Quick Look reusing the extension process: the first preview
// rendered, every later one came up blank. AppKit text has no helper
// process, so it has nothing to lose.
enum JAWHighlighter {
    static func attributedString(for source: String) -> NSAttributedString {
        let out = NSMutableAttributedString()
        let lines = source.components(separatedBy: "\n")
        for (i, line) in lines.enumerated() {
            out.append(render(line: line))
            if i < lines.count - 1 {
                out.append(NSAttributedString(string: "\n", attributes: base))
            }
        }
        return out
    }

    // MARK: - Kinds

    private enum Kind {
        case marker, step, variable, fn, deco, sep, op, num
    }

    private enum LineKind {
        case note, log, comment
    }

    private static let markerSymbols: Set<String> =
        ["^", "*", "•", "!", ">", "~", "&", "+", "-"]

    // MARK: - Line rendering

    private static func render(line: String) -> NSAttributedString {
        let (indent, rest) = splitLeadingWhitespace(line)
        let lineKind = lineLevelKind(for: rest)
        let out = NSMutableAttributedString(string: indent, attributes: base)
        tokenize(rest, lineKind: lineKind, into: out)
        return out
    }

    // Classify the whole line by its leading marker so notes/logs/comments can
    // carry the spec's emphasis (notes are red + bold, comments muted, etc.).
    private static func lineLevelKind(for rest: String) -> LineKind? {
        if rest.hasPrefix("[!]") { return .note }
        if rest.hasPrefix("[•]") { return .log }
        if rest.hasPrefix("[*]") || rest.hasPrefix("[^]") { return .comment }
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

    private static func tokenize(_ text: String, lineKind: LineKind?, into out: NSMutableAttributedString) {
        guard !text.isEmpty else { return }
        let ns = text as NSString
        let full = NSRange(location: 0, length: ns.length)
        var cursor = 0

        tokenRegex.enumerateMatches(in: text, range: full) { match, _, _ in
            guard let match = match else { return }
            let r = match.range
            if r.location > cursor {
                let gap = ns.substring(with: NSRange(location: cursor, length: r.location - cursor))
                out.append(NSAttributedString(string: gap, attributes: attributes(for: nil, in: lineKind)))
            }
            let token = ns.substring(with: r)
            out.append(NSAttributedString(string: token, attributes: attributes(for: kind(of: token, match: match), in: lineKind)))
            cursor = r.location + r.length
        }

        if cursor < ns.length {
            out.append(NSAttributedString(string: ns.substring(from: cursor), attributes: attributes(for: nil, in: lineKind)))
        }
    }

    private static func kind(of token: String, match: NSTextCheckingResult) -> Kind? {
        // Which alternation group fired tells us the kind without re-matching.
        if match.range(at: 1).location != NSNotFound { return .fn }
        if match.range(at: 2).location != NSNotFound { return bracketKind(token) }
        if match.range(at: 3).location != NSNotFound { return .deco }
        if match.range(at: 4).location != NSNotFound { return .sep }
        if match.range(at: 5).location != NSNotFound { return .op }
        if match.range(at: 6).location != NSNotFound { return .num }
        return nil
    }

    // `[42]` is a step, `[^]`/`[~]`/… are markers, everything else (`[V]`, `[ID]`)
    // is a variable reference.
    private static func bracketKind(_ token: String) -> Kind {
        let inner = String(token.dropFirst().dropLast())
        if !inner.isEmpty && inner.allSatisfy({ $0.isNumber }) { return .step }
        if markerSymbols.contains(inner) { return .marker }
        return .variable
    }

    // MARK: - Attributes

    // The same rules the CSS expressed: a token's own colour and weight, then
    // the line kind overriding it the way `.ln.note .marker` did.
    private static func attributes(for kind: Kind?, in lineKind: LineKind?) -> [NSAttributedString.Key: Any] {
        var color = Palette.fg
        var weight: NSFont.Weight = .regular
        var italic = false

        switch kind {
        case .marker:   color = Palette.marker; weight = .semibold
        case .step:     color = Palette.step;   weight = .semibold
        case .variable: color = Palette.variable; weight = .semibold
        case .fn:       color = Palette.fn
        case .deco:     color = Palette.deco
        case .sep:      color = Palette.sep
        case .op:       color = Palette.op
        case .num:      color = Palette.num
        case nil:       break
        }

        switch lineKind {
        case .note:
            color = Palette.note
            weight = .bold
        case .log:
            if kind == .marker { color = Palette.log }
        case .comment:
            italic = true
            if kind == nil || kind == .variable || kind == .marker { color = Palette.comment }
        case nil:
            break
        }

        return [
            .font: font(weight: weight, italic: italic),
            .foregroundColor: color,
        ]
    }

    private static var base: [NSAttributedString.Key: Any] {
        [.font: font(weight: .regular, italic: false), .foregroundColor: Palette.fg]
    }

    private static func font(weight: NSFont.Weight, italic: Bool) -> NSFont {
        let f = NSFont.monospacedSystemFont(ofSize: 13, weight: weight)
        guard italic else { return f }
        let descriptor = f.fontDescriptor.withSymbolicTraits(.italic)
        return NSFont(descriptor: descriptor, size: 13) ?? f
    }

    private static func splitLeadingWhitespace(_ line: String) -> (String, String) {
        guard let idx = line.firstIndex(where: { !$0.isWhitespace || $0 == "\n" }) else {
            return (line, "")
        }
        return (String(line[..<idx]), String(line[idx...]))
    }

    // MARK: - Palette

    // Tracks the VS Code extension where it can (the [•] log amber comes
    // straight from extension.ts). Each colour resolves against the current
    // appearance, so the preview follows light and dark mode live.
    private enum Palette {
        static let fg       = dynamic(light: 0x1f2328, dark: 0xd4d4d4)
        static let marker   = dynamic(light: 0x6f42c1, dark: 0xc586c0)
        static let step     = dynamic(light: 0x0969da, dark: 0x569cd6)
        static let variable = dynamic(light: 0x0a7ea4, dark: 0x4ec9b0)
        static let fn       = dynamic(light: 0x8250df, dark: 0xdcdcaa)
        static let deco     = dynamic(light: 0x953800, dark: 0xce9178)
        static let sep      = dynamic(light: 0x6e7781, dark: 0x808080)
        static let op       = dynamic(light: 0xcf222e, dark: 0xd16969)
        static let num      = dynamic(light: 0x0550ae, dark: 0xb5cea8)
        static let log      = dynamic(light: 0xb45309, dark: 0xd7ba7d)
        static let comment  = dynamic(light: 0x6e7781, dark: 0x6a9955)
        static let note     = dynamic(light: 0xcf222e, dark: 0xf14c4c)

        private static func dynamic(light: Int, dark: Int) -> NSColor {
            NSColor(name: nil) { appearance in
                let isDark = appearance.bestMatch(from: [.aqua, .darkAqua]) == .darkAqua
                return rgb(isDark ? dark : light)
            }
        }

        private static func rgb(_ hex: Int) -> NSColor {
            NSColor(
                srgbRed: CGFloat((hex >> 16) & 0xff) / 255,
                green: CGFloat((hex >> 8) & 0xff) / 255,
                blue: CGFloat(hex & 0xff) / 255,
                alpha: 1
            )
        }
    }
}
