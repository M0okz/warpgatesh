import Foundation

enum SyncAge {
  static func label(_ seconds: UInt64?) -> String {
    guard let seconds else { return "Jamais" }
    if seconds < 60 { return "À l’instant" }
    if seconds < 3600 { return "Il y a \(seconds / 60) min" }
    if seconds < 86400 { return "Il y a \(seconds / 3600) h" }
    return "Il y a \(seconds / 86400) j"
  }
}
