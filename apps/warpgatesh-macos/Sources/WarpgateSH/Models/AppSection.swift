import Foundation

enum AppSection: String, CaseIterable, Identifiable {
  case access, profiles, preferences, synchronization, logs, about
  var id: Self { self }
  var title: String {
    switch self {
    case .access: L10n.text("Accès")
    case .profiles: L10n.text("Profils")
    case .preferences: L10n.text("Préférences")
    case .synchronization: L10n.text("Synchronisation")
    case .logs: L10n.text("Journaux")
    case .about: L10n.text("À propos")
    }
  }
  var symbol: String {
    switch self {
    case .access: "server.rack"
    case .profiles: "person.2.fill"
    case .preferences: "gearshape.fill"
    case .synchronization: "arrow.clockwise"
    case .logs: "doc.text"
    case .about: "info.circle"
    }
  }
}
