import Foundation

enum AppSection: String, CaseIterable, Identifiable {
  case access, profiles, preferences, synchronization, logs, about
  var id: Self { self }
  var title: String {
    switch self {
    case .access: "Accès"
    case .profiles: "Profils"
    case .preferences: "Préférences"
    case .synchronization: "Synchronisation"
    case .logs: "Journaux"
    case .about: "À propos"
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
