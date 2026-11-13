import Foundation

struct ShellIntegration: Codable, Sendable {
  var status: String
  var path: String
  var message: String? = nil
}
