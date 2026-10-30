import AppKit
import UserNotifications

@MainActor
final class DesktopNotifications: NSObject, ObservableObject, UNUserNotificationCenterDelegate {
  @Published private(set) var authorization: UNAuthorizationStatus = .notDetermined
  @Published private(set) var message: String?
  private let center = UNUserNotificationCenter.current()
  private let settings: AppSettings
  private var ledger: NotificationLedger
  private var inFlight: Set<String> = []
  private var serverCheck: Task<Void, Never>?
  private var nextServerCheck = Date.distantPast
  private let defaults: UserDefaults
  private var pendingSection: AppSection?
  var openSection: ((AppSection) -> Void)? {
    didSet {
      if let pendingSection, let openSection {
        self.pendingSection = nil
        openSection(pendingSection)
      }
    }
  }

  init(settings: AppSettings, defaults: UserDefaults? = nil) {
    self.settings = settings
    self.defaults = defaults ?? AppSettings.defaults
    ledger = NotificationLedger(
      delivered: Set(self.defaults.stringArray(forKey: "notification-delivery-v1") ?? []))
    super.init()
    center.delegate = self
    configureCategories()
  }
  func configureCategories() {
    let categories = [
      ("token", L10n.text("Renouveler le jeton")), ("diagnostics", L10n.text("Voir le diagnostic")),
      ("update", L10n.text("Voir la mise à jour")), ("server", L10n.text("Lire les nouveautés")),
    ].map { id, title in
      UNNotificationCategory(
        identifier: id,
        actions: [
          UNNotificationAction(identifier: "open", title: L10n.text(title), options: .foreground)
        ],
        intentIdentifiers: [])
    }
    center.setNotificationCategories(Set(categories))
  }
  func refreshAuthorization() async {
    authorization = await center.notificationSettings().authorizationStatus
  }
  func enable() async {
    do {
      _ = try await center.requestAuthorization(options: [.alert])
      await refreshAuthorization()
      if authorization == .denied {
        message = L10n.text("Les notifications sont bloquées dans les réglages de macOS.")
      }
    } catch {
      message =
        L10n.text(
          "Impossible d’activer les notifications. Réessayez dans les réglages de macOS.") + " "
        + error.localizedDescription
    }
  }
  func test() async {
    await refreshAuthorization()
    guard authorization == .authorized || authorization == .provisional else {
      message = L10n.text(
        "Autorisez les notifications dans les réglages de macOS pour envoyer un test.")
      return
    }
    await deliver(
      .init(
        id: "test:\(UUID().uuidString)", title: "WarpgateSH",
        body: L10n.text(
          "Les notifications fonctionnent. Vos alertes importantes apparaîtront ici."),
        section: "preferences", category: "diagnostics"), test: true)
  }
  func evaluate(_ state: CompanionState, isDemo: Bool) async {
    guard settings.preferences.notifications.enabled else {
      serverCheck?.cancel()
      serverCheck = nil
      return
    }
    let events = NotificationPolicy.events(
      state: state, preferences: settings.preferences, now: Date())
    ledger.resolveIncidents(active: Set(events.filter { $0.id.hasPrefix("incident:") }.map(\.id)))
    persistLedger()
    await refreshAuthorization()
    for event in events { await deliver(event) }
    guard !isDemo, settings.preferences.notifications.serverUpdates,
      serverCheck == nil, Date() >= nextServerCheck
    else { return }
    nextServerCheck = Date().addingTimeInterval(6 * 3600)
    serverCheck = Task { [weak self] in
      guard let self else { return }
      defer { serverCheck = nil }
      await checkServerRelease(profiles: state.profiles)
    }
  }
  private func checkServerRelease(profiles: [CompanionProfile]) async {
    do {
      let url = URL(string: "https://api.github.com/repos/warp-tech/warpgate/releases/latest")!
      var request = URLRequest(url: url, timeoutInterval: 20)
      request.setValue("application/vnd.github+json", forHTTPHeaderField: "Accept")
      request.setValue("WarpgateSH", forHTTPHeaderField: "User-Agent")
      // This public request never contains instance URLs, user identities or Warpgate tokens.
      let (data, response) = try await URLSession.shared.data(for: request)
      try Task.checkCancellation()
      guard (response as? HTTPURLResponse)?.statusCode == 200 else { return }
      let release = try JSONDecoder().decode(ServerRelease.self, from: data)
      guard !release.prerelease, !release.draft, let latest = ReleaseVersion(release.tagName),
        settings.preferences.notifications.enabled, settings.preferences.notifications.serverUpdates
      else { return }
      for profile in profiles
      where !settings.preferences.notifications.mutedServerProfiles.contains(profile.name) {
        guard let installed = profile.warpgateVersion.flatMap(ReleaseVersion.init),
          installed < latest
        else { continue }
        await deliver(
          .init(
            id: "server:\(profile.name):\(release.tagName)",
            title: L10n.text("Nouvelle version de Warpgate"),
            body: L10n.text("%@ est disponible pour le profil %@.", release.tagName, profile.name),
            section: "about", category: "server"))
      }
    } catch {
      // Optional release checks must not interrupt SSH or synchronization.
    }
  }
  private func deliver(_ event: NotificationEvent, test: Bool = false) async {
    guard settings.preferences.notifications.enabled,
      authorization == .authorized || authorization == .provisional,
      test || ledger.shouldDeliver(event), !inFlight.contains(event.id)
    else { return }
    inFlight.insert(event.id)
    defer { inFlight.remove(event.id) }
    let content = UNMutableNotificationContent()
    content.title = event.title
    content.body = event.body
    content.categoryIdentifier = event.category
    content.userInfo = ["section": event.section, "serverRelease": event.category == "server"]
    do {
      try await center.add(
        UNNotificationRequest(identifier: event.id, content: content, trigger: nil))
      if !test {
        ledger.record(event)
        persistLedger()
      }
      message = test ? L10n.text("Notification de test envoyée.") : nil
    } catch { message = L10n.text("La notification n’a pas pu être envoyée.") }
  }
  private func persistLedger() {
    let value = Array(ledger.delivered).sorted()
    if defaults.stringArray(forKey: "notification-delivery-v1") != value {
      defaults.set(value, forKey: "notification-delivery-v1")
    }
  }
  func stop() {
    serverCheck?.cancel()
    serverCheck = nil
  }
  nonisolated func userNotificationCenter(
    _ center: UNUserNotificationCenter,
    willPresent notification: UNNotification
  ) async -> UNNotificationPresentationOptions {
    [.banner, .list]
  }
  nonisolated func userNotificationCenter(
    _ center: UNUserNotificationCenter,
    didReceive response: UNNotificationResponse
  ) async {
    let section = response.notification.request.content.userInfo["section"] as? String
    let server = response.notification.request.content.userInfo["serverRelease"] as? Bool == true
    guard response.actionIdentifier != UNNotificationDismissActionIdentifier else { return }
    await MainActor.run {
      if server {
        NSWorkspace.shared.open(
          URL(string: "https://github.com/warp-tech/warpgate/releases/latest")!)
      } else if let section, let destination = AppSection(rawValue: section) {
        if let openSection = self.openSection {
          openSection(destination)
        } else {
          self.pendingSection = destination
        }
      }
    }
  }
}
