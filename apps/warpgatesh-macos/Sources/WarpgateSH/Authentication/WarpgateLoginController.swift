import Combine
import WebKit

@MainActor
final class WarpgateLoginController: NSObject, ObservableObject, WKNavigationDelegate, WKUIDelegate
{
  let webView: WKWebView
  let origin: URL?
  @Published var currentHost = ""
  @Published var error: String?
  var authenticated: ((AuthenticatedSession) -> Void)?
  private var polling: Task<Void, Never>?
  private let expectedUsername: String?
  private let session: URLSession

  init(baseURL: String, expectedUsername: String? = nil) {
    origin = LoginOrigin.normalized(baseURL)
    self.expectedUsername = expectedUsername
    let configuration = WKWebViewConfiguration()
    configuration.websiteDataStore = .nonPersistent()
    webView = WKWebView(frame: .zero, configuration: configuration)
    let network = URLSessionConfiguration.ephemeral
    network.httpCookieStorage = nil
    network.httpShouldSetCookies = false
    network.urlCache = nil
    network.requestCachePolicy = .reloadIgnoringLocalCacheData
    session = URLSession(
      configuration: network, delegate: LoginRedirectPolicy(), delegateQueue: nil)
    super.init()
    webView.navigationDelegate = self
    webView.uiDelegate = self
    if origin == nil { error = L10n.text("Saisissez une URL HTTPS valide, sans identifiants.") }
  }
  func start() {
    guard polling == nil, let origin,
      let page = URL(string: "@warpgate/", relativeTo: origin)?.absoluteURL
    else { return }
    currentHost = origin.host ?? ""
    webView.load(URLRequest(url: page))
    polling = Task { [weak self] in
      while !Task.isCancelled {
        guard let self else { return }
        await inspectSession()
        do { try await Task.sleep(for: .seconds(1)) } catch { return }
      }
    }
  }
  private func inspectSession() async {
    guard let origin, let url = URL(string: "@warpgate/api/info", relativeTo: origin)?.absoluteURL
    else { return }
    let cookies = await webView.configuration.websiteDataStore.httpCookieStore.allCookies()
    guard let cookie = LoginOrigin.cookieHeader(cookies, for: origin) else { return }
    var request = URLRequest(url: url, timeoutInterval: 12)
    request.setValue(cookie, forHTTPHeaderField: "Cookie")
    do {
      let (bytes, response) = try await session.bytes(for: request)
      try Task.checkCancellation()
      guard (response as? HTTPURLResponse)?.statusCode == 200 else { return }
      var data = Data()
      for try await byte in bytes {
        guard data.count < 65_536 else { return }
        data.append(byte)
      }
      struct Info: Decodable { var username: String? }
      guard let username = try JSONDecoder().decode(Info.self, from: data).username,
        !username.isEmpty
      else { return }
      if let expectedUsername, username != expectedUsername {
        error = L10n.text("Connectez-vous avec l’identité %@ pour ce profil.", expectedUsername)
        return
      }
      let callback = authenticated
      authenticated = nil
      polling?.cancel()
      callback?(.init(origin: origin, username: username, cookieHeader: cookie))
    } catch is CancellationError {} catch {
      // The official login page remains interactive while authentication is incomplete.
    }
  }
  func stop() {
    polling?.cancel()
    polling = nil
    authenticated = nil
    webView.stopLoading()
    session.invalidateAndCancel()
  }
  func webView(
    _ webView: WKWebView, decidePolicyFor navigationAction: WKNavigationAction,
    decisionHandler: @escaping @MainActor (WKNavigationActionPolicy) -> Void
  ) {
    guard let url = navigationAction.request.url else {
      decisionHandler(.cancel)
      return
    }
    let allowed =
      url.scheme == "https"
      || (url.scheme == "http" && ["localhost", "127.0.0.1", "::1"].contains(url.host ?? ""))
      || url.absoluteString == "about:blank"
    if allowed, navigationAction.targetFrame?.isMainFrame != false {
      currentHost = url.host ?? currentHost
    }
    decisionHandler(allowed ? .allow : .cancel)
  }
  func webView(
    _ webView: WKWebView, createWebViewWith configuration: WKWebViewConfiguration,
    for navigationAction: WKNavigationAction, windowFeatures: WKWindowFeatures
  ) -> WKWebView? {
    if navigationAction.targetFrame == nil { webView.load(navigationAction.request) }
    return nil
  }
  func webView(
    _ webView: WKWebView, didFailProvisionalNavigation navigation: WKNavigation!,
    withError error: Error
  ) {
    if (error as NSError).code != NSURLErrorCancelled { self.error = error.localizedDescription }
  }
}
