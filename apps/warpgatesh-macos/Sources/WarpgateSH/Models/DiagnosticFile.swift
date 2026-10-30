import Foundation

struct DiagnosticFile: Codable, Sendable, Identifiable {
  var name: String
  var bytes: UInt64
  var events: UInt64
  var id: String { name }
}
