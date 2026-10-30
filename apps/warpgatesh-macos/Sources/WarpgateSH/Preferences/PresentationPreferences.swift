import SwiftUI

enum DisplayLanguage: String, Codable, CaseIterable, Identifiable {
  case automatic, french, english
  var id: Self { self }
  var label: String {
    switch self {
    case .automatic: L10n.text("Automatique")
    case .french: "Français"
    case .english: "English"
    }
  }
  var code: String {
    switch self {
    case .automatic: Locale.preferredLanguages.first?.hasPrefix("fr") == true ? "fr" : "en"
    case .french: "fr"
    case .english: "en"
    }
  }
}

enum AppearanceMode: String, Codable, CaseIterable, Identifiable {
  case warpgate, light, black, system
  var id: Self { self }
  var label: String {
    switch self {
    case .warpgate: "Warpgate"
    case .light: L10n.text("Clair")
    case .black: L10n.text("Noir")
    case .system: L10n.text("Système")
    }
  }
  var colorScheme: ColorScheme? {
    switch self {
    case .warpgate, .black: .dark
    case .light: .light
    case .system: nil
    }
  }
}

struct NotificationPreferences: Codable, Equatable {
  var enabled = false
  var token = true
  var synchronization = true
  var clientUpdates = true
  var serverUpdates = false
  var mutedServerProfiles: Set<String> = []
}

struct PresentationPreferences: Codable, Equatable {
  var language: DisplayLanguage = .automatic
  var appearance: AppearanceMode = .warpgate
  var notifications = NotificationPreferences()
  // Optional dates explicitly supplied by the user; never guessed from an API rejection.
  var tokenExpirations: [String: Date] = [:]
}
