import Foundation

enum SyncAge {
  static func label(_ seconds: UInt64?) -> String {
    guard let seconds else { return L10n.text("Jamais") }
    if seconds < 60 { return L10n.text("À l’instant") }
    if seconds < 3600 { return L10n.text("Il y a %llu min", seconds / 60) }
    if seconds < 86400 { return L10n.text("Il y a %llu h", seconds / 3600) }
    return L10n.text("Il y a %llu j", seconds / 86400)
  }
}
