import Foundation

struct AgentSetup: Codable, Equatable, Sendable {
  var needsDecision: Bool
  var enabled: Bool
  var installed: Bool = false
}
