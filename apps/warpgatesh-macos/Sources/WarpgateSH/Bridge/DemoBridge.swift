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
    update: .init(phase: "idle", channel: "unsupported", currentVersion: "0.1.17"), alerts: [])

  func request(_ command: String, args: [String: JSONValue]) async throws -> JSONValue {
    switch command {
    case "initialize": return .null
    case "get_companion_state": return try .encoding(state)
    case "sync_now":
      state.agentSynchronizing = true
      defer { state.agentSynchronizing = false }
      try await Task.sleep(for: .milliseconds(900))
      state.lastSyncAgeSeconds = 0
      return .string("Synchronisation simulée terminée.")
    case "save_preferences":
      state.preferences = try (args["preferences"] ?? .null).decoded()
      for i in state.profiles.indices {
        state.profiles[i].isDefault = state.profiles[i].name == state.preferences.defaultProfile
      }
      return .null
    case "preview_diagnostics":
      return try .encoding(
        DiagnosticsPreview(
          logDirectory: "Journaux de démonstration", retentionDays: 7, totalBytes: 2048,
          totalEvents: 12, files: [.init(name: "agent-demo.jsonl", bytes: 2048, events: 12)]))
    case "check_for_updates":
      state.update.phase = "current"
      return try .encoding(state.update)
    default:
      throw BridgeError.message(
        "Cette action nécessite l’application connectée. Aucune modification réelle n’a été effectuée."
      )
    }
  }
  func stop() {}
}
