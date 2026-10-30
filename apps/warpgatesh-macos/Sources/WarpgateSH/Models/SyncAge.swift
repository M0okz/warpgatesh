import Foundation

enum SyncAge {
  static func label(_ seconds: UInt64?, language: DisplayLanguage = L10n.language) -> String {
    func text(_ key: String, _ arguments: CVarArg...) -> String {
      let translated = L10n.translation(key, language: language)
      return arguments.isEmpty
        ? translated
        : String(
          format: translated, locale: Locale(identifier: language.code), arguments: arguments)
    }
    guard let seconds else { return text("Jamais") }
    if seconds < 60 { return text("À l’instant") }
    if seconds < 3600 { return text("Il y a %llu min", seconds / 60) }
    if seconds < 86400 { return text("Il y a %llu h", seconds / 3600) }
    return text("Il y a %llu j", seconds / 86400)
  }
}
