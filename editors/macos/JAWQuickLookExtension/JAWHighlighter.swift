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
    /// Background the text should sit on — the theme's own, not the system's.
    static var backgroundColor: NSColor { Palette.bg }

    /// `dark` picks the palette's variant. Colours are dynamic either way; the
    /// flag decides weights, which cannot be.
    static func attributedString(for source: String, dark: Bool) -> NSAttributedString {
        let out = NSMutableAttributedString()
        let lines = source.components(separatedBy: "\n")
        for (i, line) in lines.enumerated() {
            out.append(render(line: line, dark: dark))
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

    private static func render(line: String, dark: Bool) -> NSAttributedString {
        let (indent, rest) = splitLeadingWhitespace(line)
        let lineKind = lineLevelKind(for: rest)
        let out = NSMutableAttributedString(string: indent, attributes: base)
        tokenize(rest, lineKind: lineKind, dark: dark, into: out)
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

    private static func tokenize(_ text: String, lineKind: LineKind?, dark: Bool, into out: NSMutableAttributedString) {
        guard !text.isEmpty else { return }
        let ns = text as NSString
        let full = NSRange(location: 0, length: ns.length)
        var cursor = 0

        tokenRegex.enumerateMatches(in: text, range: full) { match, _, _ in
            guard let match = match else { return }
            let r = match.range
            if r.location > cursor {
                let gap = ns.substring(with: NSRange(location: cursor, length: r.location - cursor))
                out.append(NSAttributedString(string: gap, attributes: attributes(for: nil, in: lineKind, dark: dark)))
            }
            let token = ns.substring(with: r)
            out.append(NSAttributedString(string: token, attributes: attributes(for: kind(of: token, match: match), in: lineKind, dark: dark)))
            cursor = r.location + r.length
        }

        if cursor < ns.length {
            out.append(NSAttributedString(string: ns.substring(from: cursor), attributes: attributes(for: nil, in: lineKind, dark: dark)))
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

    // Future Earth (dishmint/theme-depot), by role: markers are keywords,
    // function refs are functions, variables are types, steps and numbers are
    // constants, decorators are attributes, the em dash is punctuation. The
    // theme's rule decides the rest — dark mode tells things apart by colour,
    // light mode by weight with as little colour as it can get away with — and
    // the line kind overrides both, as the CSS's `.ln.note .marker` did.
    private static func attributes(for kind: Kind?, in lineKind: LineKind?, dark: Bool) -> [NSAttributedString.Key: Any] {
        var color = Palette.fg
        var weight: NSFont.Weight = .regular
        var italic = false

        switch kind {
        case .marker:   color = Palette.red;   weight = .semibold
        case .step:     color = dark ? Palette.gold : Palette.fg; weight = .semibold
        case .variable: color = Palette.blue;  weight = dark ? .regular : .bold
        case .fn:       color = dark ? Palette.yellow : Palette.fg; weight = dark ? .regular : .bold
        case .deco:     color = Palette.stone
        case .sep:      color = Palette.fgDim
        case .op:       color = Palette.fg
        case .num:      color = dark ? Palette.gold : Palette.fg
        case nil:       break
        }

        switch lineKind {
        case .note:
            color = Palette.redBright
            weight = .bold
        case .log:
            if kind == .marker { color = Palette.yellow }
        case .comment:
            color = Palette.comment
            italic = true
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

    // Future Earth, light and dark, from dishmint/theme-depot
    // (helix/future-earth/*.toml). Each colour resolves against the current
    // appearance, so the preview follows light and dark mode live.
    private enum Palette {
        static let bg        = dynamic(light: 0xFBFFFE, dark: 0x1B1B1E)
        static let fg        = dynamic(light: 0x1B1B1E, dark: 0xFBFFFE)
        static let fgDim     = dynamic(light: 0x797470, dark: 0xB0ACA7)
        static let comment   = dynamic(light: 0xB0B0AE, dark: 0x5C5C60)
        static let red       = dynamic(light: 0xB54B4B, dark: 0xE06B6B)
        static let redBright = dynamic(light: 0xD93636, dark: 0xFF4D4D)
        static let yellow    = dynamic(light: 0xA8740D, dark: 0xFAB025)
        static let gold      = dynamic(light: 0xA8740D, dark: 0xD4A04A)
        static let blue      = dynamic(light: 0x2D5A6B, dark: 0x4F8BA0)
        static let stone     = dynamic(light: 0x797470, dark: 0x9A9590)

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
