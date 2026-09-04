import Cocoa
import Quartz

// Renders a .jaw file as syntax-highlighted text in an NSTextView. Quick Look
// instantiates this class (named in Info.plist) for each preview request.
//
// Not a WKWebView: WebKit's content process does not survive Quick Look
// reusing the extension process, so the first preview rendered and every one
// after came up blank. An AppKit text view has no helper process to lose.
class PreviewViewController: NSViewController, QLPreviewingController {
    private var textView: NSTextView!

    override func loadView() {
        let scroll = NSScrollView(frame: NSRect(x: 0, y: 0, width: 800, height: 600))
        scroll.hasVerticalScroller = true
        scroll.hasHorizontalScroller = true
        scroll.autohidesScrollers = true
        scroll.borderType = .noBorder
        scroll.drawsBackground = true
        scroll.backgroundColor = JAWHighlighter.backgroundColor

        let text = NSTextView(frame: scroll.bounds)
        text.isEditable = false
        text.isSelectable = true
        text.drawsBackground = true
        text.backgroundColor = JAWHighlighter.backgroundColor
        text.textContainerInset = NSSize(width: 20, height: 16)

        // Code: no wrapping. The container grows with the longest line and the
        // scroll view supplies the horizontal scroller.
        text.isHorizontallyResizable = true
        text.isVerticallyResizable = true
        text.autoresizingMask = [.width]
        text.minSize = .zero
        text.maxSize = NSSize(width: CGFloat.greatestFiniteMagnitude, height: CGFloat.greatestFiniteMagnitude)
        text.textContainer?.widthTracksTextView = false
        text.textContainer?.containerSize = NSSize(width: CGFloat.greatestFiniteMagnitude, height: CGFloat.greatestFiniteMagnitude)

        scroll.documentView = text
        self.textView = text
        self.view = scroll
    }

    func preparePreviewOfFile(at url: URL) async throws {
        _ = view  // force loadView; Quick Look may call this before it ever asks for the view
        let data = try Data(contentsOf: url)
        // .jaw is UTF-8 by spec (em dashes, the [•] marker); fall back to a lossy
        // decode rather than failing the preview outright.
        let source = String(data: data, encoding: .utf8)
            ?? String(decoding: data, as: UTF8.self)
        let dark = view.effectiveAppearance.bestMatch(from: [.aqua, .darkAqua]) == .darkAqua
        textView.textStorage?.setAttributedString(JAWHighlighter.attributedString(for: source, dark: dark))
    }
}
