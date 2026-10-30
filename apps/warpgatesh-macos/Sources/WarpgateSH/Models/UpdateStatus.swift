import Foundation

struct UpdateStatus: Codable, Sendable {
  var phase: String
  var channel: String
  var currentVersion: String
  var availableVersion: String?
  var notes: String?
  var checkedAtEpochSeconds: UInt64?
  var progressPercent: UInt8?
  var message: String?
  var isBusy: Bool { ["checking", "downloading", "installing"].contains(phase) }
}
