import Foundation

struct ProfileInspection: Codable, Sendable {
  var normalizedBaseUrl: String
  var username: String
  var warpgateVersion: String?
  var sshHost: String
  var sshPort: UInt16
  var fingerprints: String
}
