import Foundation

struct ManagedTokenStatus: Codable, Sendable, Equatable {
  var automaticRenewal: Bool
  var lastError: String? = nil
  var expiresAtEpochSeconds: UInt64?
  var renewAfterEpochSeconds: UInt64?
}
