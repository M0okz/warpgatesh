import Foundation

struct CompanionState: Codable, Sendable {
  var agentSetup: AgentSetup? = nil
  var agentRunning: Bool
  var agentSynchronizing: Bool
  var profiles: [CompanionProfile]
  var targets: [CompanionTarget]
  var lastSyncAgeSeconds: UInt64?
  var preferences: CompanionPreferences
  var terminalIntegration: TerminalIntegration
  var update: UpdateStatus
  var alerts: [CompanionAlert]
  var manualMode: Bool { agentSetup?.installed == false && !agentRunning }
}
