import Cocoa
import Quartz
import WebKit

// Renders a .jaw file as syntax-highlighted HTML inside a WKWebView. Quick Look
// instantiates this class (named in Info.plist) for each preview request.
class PreviewViewController: NSViewController, QLPreviewingController, WKNavigationDelegate {
    private var webView: WKWebView!

    override func loadView() {
        let configuration = WKWebViewConfiguration()
        let web = WKWebView(frame: .zero, configuration: configuration)
        web.navigationDelegate = self
        self.webView = web
        self.view = web
    }

    func preparePreviewOfFile(at url: URL) async throws {
        let data = try Data(contentsOf: url)
        // .jaw is UTF-8 by spec (em dashes, the [•] marker); fall back to a lossy
        // decode rather than failing the preview outright.
        let source = String(data: data, encoding: .utf8)
            ?? String(decoding: data, as: UTF8.self)
        let html = JAWHighlighter.html(for: source)

        // Start the load and return. Quick Look hosts this view live and paints
        // it as it renders, so there is nothing to gain by waiting for
        // `didFinish` — and the spinner stays up until this method returns, so
        // waiting on a delegate callback that may never arrive in the
        // extension's sandbox hangs the preview. Finishing the page matters
        // for thumbnail capture, which this extension does not provide.
        webView.loadHTMLString(html, baseURL: nil)
    }

    func webView(_ webView: WKWebView, didFail navigation: WKNavigation!, withError error: Error) {
        NSLog("JAWQuickLook: preview load failed: %@", error.localizedDescription)
    }

    func webView(_ webView: WKWebView,
                 didFailProvisionalNavigation navigation: WKNavigation!,
                 withError error: Error) {
        NSLog("JAWQuickLook: preview load failed: %@", error.localizedDescription)
    }
}
