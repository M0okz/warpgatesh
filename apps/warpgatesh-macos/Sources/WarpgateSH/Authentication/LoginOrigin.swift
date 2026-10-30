import Foundation

enum LoginOrigin {
  static func normalized(_ text: String) -> URL? {
    guard
      var components = URLComponents(string: text.trimmingCharacters(in: .whitespacesAndNewlines)),
      let host = components.host?.lowercased(), !host.isEmpty,
      components.user == nil, components.password == nil,
      components.scheme == "https"
        || (components.scheme == "http" && ["localhost", "127.0.0.1", "::1"].contains(host))
    else { return nil }
    components.query = nil
    components.fragment = nil
    if let range = components.path.range(of: "/@warpgate") {
      components.path = String(components.path[..<range.lowerBound]) + "/"
    } else if !components.path.hasSuffix("/") {
      components.path += "/"
    }
    return components.url
  }
  static func cookieHeader(_ cookies: [HTTPCookie], for origin: URL) -> String? {
    guard let host = origin.host?.lowercased(),
      let api = URL(string: "@warpgate/api/info", relativeTo: origin)?.absoluteURL
    else { return nil }
    let selected = cookies.filter { cookie in
      let domain = cookie.domain.lowercased()
      let matches =
        domain == host
        || (domain.hasPrefix(".") && (host == String(domain.dropFirst()) || host.hasSuffix(domain)))
      return cookie.name == "warpgate-http-session" && matches
        && api.path.hasPrefix(cookie.path)
        && (cookie.expiresDate == nil || cookie.expiresDate! > Date())
        && (!cookie.isSecure || origin.scheme == "https")
        && !cookie.value.contains(where: { "\r\n;".contains($0) })
    }.sorted { $0.path.count > $1.path.count }
    guard let cookie = selected.first else { return nil }
    return "warpgate-http-session=" + cookie.value
  }
}
