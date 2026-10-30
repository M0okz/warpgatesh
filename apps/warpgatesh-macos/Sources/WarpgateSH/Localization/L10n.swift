import Foundation

enum L10n {
  static var language: DisplayLanguage {
    let defaults =
      CommandLine.arguments.contains("--demo")
      ? UserDefaults(suiteName: "dev.warpgatesh.preview.settings")! : .standard
    guard let data = defaults.data(forKey: "presentation-preferences-v1"),
      let value = try? JSONDecoder().decode(PresentationPreferences.self, from: data)
    else { return .automatic }
    return value.language
  }
  static func text(_ key: String, _ arguments: CVarArg...) -> String {
    let translated = translation(key, language: language)
    return arguments.isEmpty
      ? translated
      : String(format: translated, locale: Locale(identifier: language.code), arguments: arguments)
  }
  static func translation(_ key: String, language: DisplayLanguage) -> String {
    guard let path = Bundle.module.path(forResource: language.code, ofType: "lproj"),
      let bundle = Bundle(path: path)
    else { return key }
    return bundle.localizedString(forKey: key, value: key, table: "Localizable")
  }
}
