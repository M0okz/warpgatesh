import Foundation

struct CompanionPreferences: Codable, Sendable, Equatable {
  var syncIntervalSeconds: UInt64
  var launchCompanionAtLogin: Bool
  var defaultProfile: String?
}
