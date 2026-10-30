import Foundation

struct NotificationEvent: Equatable, Sendable {
  let id: String
  let title: String
  let body: String
  let section: String
  let category: String
}

struct NotificationLedger {
  var delivered: Set<String> = []
  mutating func resolveIncidents(active: Set<String>) {
    delivered = delivered.filter { !$0.hasPrefix("incident:") || active.contains($0) }
  }
  func shouldDeliver(_ event: NotificationEvent) -> Bool { !delivered.contains(event.id) }
  mutating func record(_ event: NotificationEvent) { delivered.insert(event.id) }
}

enum NotificationPolicy {
  static func events(state: CompanionState, preferences: PresentationPreferences, now: Date)
    -> [NotificationEvent]
  {
    let options = preferences.notifications
    guard options.enabled else { return [] }
    var events: [NotificationEvent] = []
    for alert in state.alerts {
      let token = alert.id == "token"
      guard token ? options.token : options.synchronization else { continue }
      events.append(
        .init(
          id: "incident:\(alert.id)", title: L10n.text(alert.title),
          body: token
            ? L10n.text("Reconnectez votre profil pour rétablir la synchronisation.")
            : L10n.text("La synchronisation nécessite votre attention. Consultez le diagnostic."),
          section: token || alert.action == "profiles" ? "profiles" : "logs",
          category: token ? "token" : "diagnostics"))
    }
    if options.token {
      for profile in state.profiles {
        guard let expiry = preferences.tokenExpirations[profile.name] else { continue }
        let remaining = expiry.timeIntervalSince(now)
        guard remaining <= 7 * 86400 else { continue }
        let threshold = remaining <= 0 ? "expired" : remaining <= 86400 ? "day" : "week"
        events.append(
          .init(
            id: "expiry:\(profile.name):\(Int(expiry.timeIntervalSince1970)):\(threshold)",
            title: L10n.text(
              remaining <= 0 ? L10n.text("Jeton expiré") : L10n.text("Jeton bientôt expiré")),
            body: L10n.text("Vérifiez le jeton du profil %@.", profile.name), section: "profiles",
            category: "token"))
      }
    }
    if options.clientUpdates, let version = state.update.availableVersion,
      state.update.phase == "available"
    {
      events.append(
        .init(
          id: "client:\(version)", title: L10n.text("Mise à jour de WarpgateSH"),
          body: L10n.text("La version %@ est disponible.", version), section: "about",
          category: "update"))
    }
    return events
  }
}
