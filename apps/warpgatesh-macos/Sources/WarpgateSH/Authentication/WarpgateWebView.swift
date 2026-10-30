import SwiftUI
import WebKit

struct WarpgateWebView: NSViewRepresentable {
  let controller: WarpgateLoginController
  func makeNSView(context: Context) -> WKWebView { controller.webView }
  func updateNSView(_ nsView: WKWebView, context: Context) {}
}
