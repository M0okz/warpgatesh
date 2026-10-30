import Foundation

struct CompanionProfile: Codable, Sendable, Identifiable {
  var name: String
  var username: String
  var baseUrl: String
  var warpgateVersion: String?
  var sshHost: String
  var sshPort: UInt16
  var isDefault: Bool
  var id: String { name }
}
