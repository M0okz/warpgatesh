import Foundation
import Testing

@testable import WarpgateSH

struct AuthenticationTests {
  @Test func rejectsUnsafeOriginsAndNormalizesCopiedPage() {
    #expect(LoginOrigin.normalized("https://alice:password@example.test/") == nil)
    #expect(LoginOrigin.normalized("http://example.test/") == nil)
    #expect(LoginOrigin.normalized("file:///tmp/login") == nil)
    #expect(
      LoginOrigin.normalized("https://example.test/prefix/@warpgate/#/profile")?.absoluteString
        == "https://example.test/prefix/")
  }
  @Test func onlySelectsTheMatchingWarpgateSession() throws {
    func cookie(_ name: String, _ domain: String, _ path: String = "/") throws -> HTTPCookie {
      try #require(
        HTTPCookie(properties: [
          .name: name, .value: "test-secret", .domain: domain, .path: path, .secure: "TRUE",
        ]))
    }
    let origin = try #require(URL(string: "https://warpgate.example.test/"))
    #expect(
      LoginOrigin.cookieHeader(
        [
          try cookie("session", "warpgate.example.test"),
          try cookie("warpgate-http-session", "sso.example.test"),
          try cookie("warpgate-http-session", "warpgate.example.test", "/other"),
        ], for: origin) == nil)
    #expect(
      LoginOrigin.cookieHeader(
        [try cookie("warpgate-http-session", "warpgate.example.test")], for: origin)
        == "warpgate-http-session=test-secret")
    #expect(
      LoginOrigin.cookieHeader(
        [try cookie("warpgate-http-session", ".example.test")],
        for: URL(string: "https://evil-example.test/")!) == nil)
  }
  @Test @MainActor func demoManagedTokensAreOnlyInMemory() async throws {
    let demo = DemoBridge()
    let status: ManagedTokenStatus = try await demo.fetch(
      "rotate_profile_token", args: ["name": .string("homelab")])
    #expect(status.automaticRenewal)
    #expect(status.expiresAtEpochSeconds != nil)
    try await demo.perform(
      "set_token_auto_renew", args: ["name": .string("homelab"), "enabled": .bool(false)])
    #expect(demo.state.profiles[0].managedToken?.automaticRenewal == false)
    let fresh = DemoBridge()
    #expect(fresh.state.profiles[0].managedToken == nil)
    await #expect(throws: BridgeError.self) { try await demo.perform("uninstall_warpgatesh") }
  }
  @Test @MainActor func serverExpiryOverridesAnOldManualReminder() {
    let now = Date(timeIntervalSince1970: 2_000_000_000)
    var state = DemoBridge().state
    var preferences = PresentationPreferences()
    preferences.notifications.enabled = true
    preferences.tokenExpirations["homelab"] = now.addingTimeInterval(-86400)
    state.profiles[0].managedToken = .init(
      automaticRenewal: true, expiresAtEpochSeconds: 2_000_000_000 + 30 * 86400,
      renewAfterEpochSeconds: 2_000_000_000 + 24 * 86400)
    #expect(NotificationPolicy.events(state: state, preferences: preferences, now: now).isEmpty)
    state.profiles[0].managedToken?.expiresAtEpochSeconds = 2_000_000_000 + 3600
    #expect(
      NotificationPolicy.events(state: state, preferences: preferences, now: now).first?.id
        .hasSuffix(":day") == true)
  }

}
