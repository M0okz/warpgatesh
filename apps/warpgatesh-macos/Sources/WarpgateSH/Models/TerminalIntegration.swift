import Foundation

struct TerminalIntegration: Codable, Sendable {
  var status: String
  var path: String
  var zsh: ShellIntegration? = nil
  var commandAvailable: Bool { ["managed", "external"].contains(status) }
}
