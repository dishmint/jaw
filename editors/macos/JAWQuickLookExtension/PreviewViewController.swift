import Cocoa
import Quartz
import WebKit

// Renders a .jaw file as syntax-highlighted HTML inside a WKWebView. Quick Look
// instantiates this class (named in Info.plist) for each preview request.
class PreviewViewController: NSViewController, QLPreviewingController, WKNavigationDelegate {
    private var webView: WKWebView!
    private var loadContinuation: CheckedContinuation<Void, Error>?

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

        // Wait for the page to finish loading so Quick Look captures the rendered
        // result, not a blank web view.
        try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<Void, Error>) in
            self.loadContinuation = continuation
            self.webView.loadHTMLString(html, baseURL: nil)
        }
    }

    func webView(_ webView: WKWebView, didFinish navigation: WKNavigation!) {
        loadContinuation?.resume()
        loadContinuation = nil
    }

    func webView(_ webView: WKWebView, didFail navigation: WKNavigation!, withError error: Error) {
        loadContinuation?.resume(throwing: error)
        loadContinuation = nil
    }

    func webView(_ webView: WKWebView,
                 didFailProvisionalNavigation navigation: WKNavigation!,
                 withError error: Error) {
        loadContinuation?.resume(throwing: error)
        loadContinuation = nil
    }
}
