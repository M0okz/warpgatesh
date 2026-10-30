import AppKit
import Combine

@MainActor
final class AppSettings: ObservableObject {
  static let storageKey = "presentation-preferences-v1"
  static var defaults: UserDefaults {
    CommandLine.arguments.contains("--demo")
      ? UserDefaults(suiteName: "dev.warpgatesh.preview.settings")! : .standard
  }
  @Published var preferences: PresentationPreferences {
    didSet {
      if let data = try? JSONEncoder().encode(preferences) {
        defaults.set(data, forKey: Self.storageKey)
      }
      applyAppearance()
    }
  }
  private let defaults: UserDefaults
  init(defaults: UserDefaults? = nil) {
    self.defaults = defaults ?? Self.defaults
    preferences = Self.load(from: self.defaults)
  }
  static func load(from defaults: UserDefaults) -> PresentationPreferences {
    guard let data = defaults.data(forKey: storageKey),
      let preferences = try? JSONDecoder().decode(PresentationPreferences.self, from: data)
    else { return PresentationPreferences() }
    return preferences
  }
  func applyAppearance() {
    switch preferences.appearance {
    case .warpgate, .black: NSApp?.appearance = NSAppearance(named: .darkAqua)
    case .light: NSApp?.appearance = NSAppearance(named: .aqua)
    case .system: NSApp?.appearance = nil
    }
  }
}
