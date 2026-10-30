import Foundation

struct ProfileRequest: Codable, Sendable, Equatable {
  var name = ""
  var baseUrl = ""
  var token = ""
  var sessionCookie: String?
  var sshHost: String?
  var sshPort: UInt16?
  var expectedFingerprints: String?
}
