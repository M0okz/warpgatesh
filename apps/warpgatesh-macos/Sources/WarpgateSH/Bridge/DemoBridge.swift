import Foundation

@MainActor
final class DemoBridge: CompanionTransport {
  let isDemo = true
  var state = CompanionState(
    agentRunning: true, agentSynchronizing: false,
    profiles: [
      .init(
        name: "homelab", username: "demo", baseUrl: "https://warpgate.example.com/",
        warpgateVersion: "0.27.1", sshHost: "warpgate.example.com", sshPort: 2222, isDefault: true)
    ],
    targets: ([
      "app-01", "build-01", "db-primary", "proxy-01", "worker-01",
    ] + (1...49).map { String(format: "sandbox-node-%02d", $0) }).map {
      .init(alias: $0, qualifiedAlias: "\($0).homelab", name: $0, profile: "homelab")
    }, lastSyncAgeSeconds: 180,
    preferences: .init(
      syncIntervalSeconds: 300, launchCompanionAtLogin: false, defaultProfile: "homelab"),
    terminalIntegration: .init(status: "missing", path: "/usr/local/bin/warpgatesh"),
    update: .init(
      phase: "idle", channel: "unsupported",
      currentVersion: Bundle.main.object(forInfoDictionaryKey: "CFBundleShortVersionString")
        as? String ?? "development"), alerts: [])

  init(firstLaunch: Bool = CommandLine.arguments.contains("--demo-first-launch")) {
    state.agentSetup = .init(
      needsDecision: firstLaunch, enabled: !firstLaunch, installed: !firstLaunch)
    state.agentRunning = !firstLaunch
  }

  func request(_ command: String, args: [String: JSONValue]) async throws -> JSONValue {
    switch command {
    case "initialize": return try .encoding(state.agentSetup)
    case "set_background_agent":
      let enabled: Bool = try (args["enabled"] ?? .null).decoded()
      state.agentSetup = .init(needsDecision: false, enabled: enabled, installed: enabled)
      state.agentRunning = enabled
      return try .encoding(state.agentSetup)
    case "get_companion_state": return try .encoding(state)
    case "sync_now":
      state.agentSynchronizing = true
      defer { state.agentSynchronizing = false }
      try await Task.sleep(for: .milliseconds(900))
      state.lastSyncAgeSeconds = 0
      return .string(L10n.text("Synchronisation simulée terminée."))
    case "save_preferences":
      state.preferences = try (args["preferences"] ?? .null).decoded()
      for i in state.profiles.indices {
        state.profiles[i].isDefault = state.profiles[i].name == state.preferences.defaultProfile
      }
      return .null
    case "inspect_profile":
      let request: ProfileRequest = try (args["request"] ?? .null).decoded()
      return try .encoding(
        ProfileInspection(
          normalizedBaseUrl: request.baseUrl, username: "demo", warpgateVersion: "0.30.2",
          sshHost: "warpgate.example.com", sshPort: 2222,
          fingerprints: "SHA256:DEMO-ONLY-NOT-A-REAL-HOST-KEY"))
    case "add_profile":
      let request: ProfileRequest = try (args["request"] ?? .null).decoded()
      guard request.expectedFingerprints == "SHA256:DEMO-ONLY-NOT-A-REAL-HOST-KEY" else {
        throw BridgeError.message("Demo: approve the simulated SSH fingerprint first.")
      }
      state.profiles.removeAll { $0.name == request.name }
      state.profiles.append(
        .init(
          name: request.name, username: "demo", baseUrl: request.baseUrl, warpgateVersion: "0.30.2",
          sshHost: "warpgate.example.com", sshPort: 2222, isDefault: false,
          managedToken: request.sessionCookie == nil ? nil : demoToken()))
      return .null
    case "rotate_profile_token":
      let name: String = try (args["name"] ?? .null).decoded()
      guard let index = state.profiles.firstIndex(where: { $0.name == name }) else {
        throw BridgeError.message("Demo: unknown profile.")
      }
      let status = demoToken()
      state.profiles[index].managedToken = status
      return try .encoding(status)
    case "set_token_auto_renew":
      let name: String = try (args["name"] ?? .null).decoded()
      let enabled: Bool = try (args["enabled"] ?? .null).decoded()
      guard let index = state.profiles.firstIndex(where: { $0.name == name }),
        state.profiles[index].managedToken != nil
      else { throw BridgeError.message("Demo: unknown managed profile.") }
      state.profiles[index].managedToken?.automaticRenewal = enabled
      return .null
    case "preview_diagnostics":
      return try .encoding(
        DiagnosticsPreview(
          logDirectory: L10n.text("Journaux de démonstration"), retentionDays: 7, totalBytes: 2048,
          totalEvents: 12, files: [.init(name: "agent-demo.jsonl", bytes: 2048, events: 12)]))
    case "check_for_updates":
      state.update.phase = "current"
      return try .encoding(state.update)
    default:
      throw BridgeError.message(
        L10n.text(
          "Cette action nécessite l’application connectée. Aucune modification réelle n’a été effectuée."
        )
      )
    }
  }
  private func demoToken() -> ManagedTokenStatus {
    let now = UInt64(Date().timeIntervalSince1970)
    return .init(
      automaticRenewal: true, expiresAtEpochSeconds: now + 30 * 86400,
      renewAfterEpochSeconds: now + 24 * 86400)
  }
  func stop() {}
}
