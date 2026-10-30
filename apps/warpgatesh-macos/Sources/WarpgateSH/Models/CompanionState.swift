import Foundation

struct CompanionState: Codable, Sendable {
  var agentRunning: Bool
  var agentSynchronizing: Bool
  var profiles: [CompanionProfile]
  var targets: [CompanionTarget]
  var lastSyncAgeSeconds: UInt64?
  var preferences: CompanionPreferences
  var terminalIntegration: TerminalIntegration
  var update: UpdateStatus
  var alerts: [CompanionAlert]
}
