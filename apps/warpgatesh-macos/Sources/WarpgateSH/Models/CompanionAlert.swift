import Foundation

struct CompanionAlert: Codable, Sendable, Identifiable {
  var id: String
  var kind: String
  var title: String
  var message: String
  var action: String?
}
