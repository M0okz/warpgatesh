import Foundation

struct DiagnosticsPreview: Codable, Sendable {
  var logDirectory: String
  var retentionDays: UInt8
  var totalBytes: UInt64
  var totalEvents: UInt64
  var files: [DiagnosticFile]
}
