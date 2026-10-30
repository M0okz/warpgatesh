import Foundation

struct CompanionTarget: Codable, Sendable, Identifiable, Equatable {
  var alias: String
  var qualifiedAlias: String
  var name: String
  var profile: String
  var id: String { qualifiedAlias }
  static func matching(_ query: String, in targets: [Self]) -> [Self] {
    let term = query.trimmingCharacters(in: .whitespacesAndNewlines)
    guard !term.isEmpty else { return targets }
    return targets.filter {
      $0.name.localizedStandardContains(term) || $0.alias.localizedStandardContains(term)
        || $0.qualifiedAlias.localizedStandardContains(term)
        || $0.profile.localizedStandardContains(term)
    }
  }
}
