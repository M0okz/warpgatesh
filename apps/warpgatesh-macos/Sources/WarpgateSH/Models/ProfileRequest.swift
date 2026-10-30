import Foundation

struct ProfileRequest: Codable, Sendable, Equatable {
  var name = ""
  var baseUrl = ""
  var token = ""
  var sshHost: String?
  var sshPort: UInt16?
  var expectedFingerprints: String?
}
