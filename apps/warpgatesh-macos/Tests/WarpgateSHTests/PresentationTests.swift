import Foundation
import SwiftUI
import Testing

@testable import WarpgateSH

struct PresentationTests {
  @Test @MainActor func settingsPersistWithoutChangingBackendPreferences() throws {
    let suite = "warpgatesh-settings-test-\(UUID().uuidString)"
    let defaults = try #require(UserDefaults(suiteName: suite))
    defer { defaults.removePersistentDomain(forName: suite) }
    let settings = AppSettings(defaults: defaults)
    #expect(settings.preferences.appearance == .warpgate)
    #expect(!settings.preferences.notifications.enabled)
    settings.preferences.language = .english
    settings.preferences.appearance = .light
    settings.preferences.notifications.enabled = true
    settings.preferences.notifications.mutedServerProfiles = ["work"]
    let restored = AppSettings(defaults: defaults)
    #expect(restored.preferences == settings.preferences)
    #expect(defaults.object(forKey: "syncIntervalSeconds") == nil)
    defaults.set(Data("invalid-json".utf8), forKey: AppSettings.storageKey)
    #expect(AppSettings.load(from: defaults) == PresentationPreferences())
  }
  @Test func bothLanguagesContainAllMessages() throws {
    let frenchURL = try #require(
      Bundle.module.url(
        forResource: "Localizable", withExtension: "strings", subdirectory: "fr.lproj"))
    let englishURL = try #require(
      Bundle.module.url(
        forResource: "Localizable", withExtension: "strings", subdirectory: "en.lproj"))
    let french = try #require(NSDictionary(contentsOf: frenchURL) as? [String: String])
    let english = try #require(NSDictionary(contentsOf: englishURL) as? [String: String])
    #expect(french.count >= 200)
    #expect(Set(french.keys) == Set(english.keys))
    #expect(L10n.translation("Préférences", language: .english) == "Preferences")
    #expect(L10n.translation("Préférences", language: .french) == "Préférences")
    #expect(
      L10n.translation("Supprimer le profil « %@ » ?", language: .english) == "Remove profile “%@”?"
    )
  }
  @Test func appearanceModesKeepSystemAutomatic() {
    #expect(AppearanceMode.warpgate.colorScheme == .dark)
    #expect(AppearanceMode.black.colorScheme == .dark)
    #expect(AppearanceMode.light.colorScheme == .light)
    #expect(AppearanceMode.system.colorScheme == nil)
  }
  @Test func stableServerVersionsUseNumericComparison() throws {
    #expect(try #require(ReleaseVersion("v0.30.2")) > #require(ReleaseVersion("0.9.12")))
    #expect(try #require(ReleaseVersion("1.0.0")) > #require(ReleaseVersion("0.99.99")))
    #expect(ReleaseVersion("0.30.0-beta.1") == nil)
    #expect(ReleaseVersion("garbage") == nil)
    #expect(ReleaseVersion("0.30") == nil)
  }
  @Test @MainActor func remindersRespectDatesAndNotificationChoices() {
    let now = Date(timeIntervalSince1970: 2_000_000_000)
    let state = DemoBridge().state
    var preferences = PresentationPreferences()
    preferences.notifications.enabled = true
    #expect(NotificationPolicy.events(state: state, preferences: preferences, now: now).isEmpty)
    preferences.tokenExpirations["homelab"] = now.addingTimeInterval(5 * 86400)
    let week = NotificationPolicy.events(state: state, preferences: preferences, now: now)
    #expect(week.count == 1)
    #expect(week.first?.id.hasSuffix(":week") == true)
    #expect(
      NotificationPolicy.events(
        state: state, preferences: preferences, now: now.addingTimeInterval(4.5 * 86400)
      ).first?.id.hasSuffix(":day") == true)
    #expect(
      NotificationPolicy.events(
        state: state, preferences: preferences, now: now.addingTimeInterval(6 * 86400)
      ).first?.id.hasSuffix(":expired") == true)
    preferences.notifications.token = false
    #expect(NotificationPolicy.events(state: state, preferences: preferences, now: now).isEmpty)
    preferences.notifications.enabled = false
    #expect(NotificationPolicy.events(state: state, preferences: preferences, now: now).isEmpty)
  }
  @Test @MainActor func incidentsRecoverAndUpdatesAreNotRepeated() {
    var state = DemoBridge().state
    state.alerts = [
      .init(
        id: "token", kind: "error", title: "Jeton refusé", message: "secret must never appear",
        action: "profiles")
    ]
    state.update.phase = "available"
    state.update.availableVersion = "0.1.99"
    var preferences = PresentationPreferences()
    preferences.notifications.enabled = true
    let events = NotificationPolicy.events(state: state, preferences: preferences, now: Date())
    #expect(events.count == 2)
    #expect(events.first?.section == "profiles")
    #expect(events.allSatisfy { !$0.body.contains("secret") })
    var ledger = NotificationLedger()
    for event in events {
      #expect(ledger.shouldDeliver(event))
      ledger.record(event)
      #expect(!ledger.shouldDeliver(event))
    }
    ledger.resolveIncidents(active: [])
    #expect(ledger.shouldDeliver(events[0]))
    #expect(!ledger.shouldDeliver(events[1]))
    preferences.notifications.token = false
    #expect(
      NotificationPolicy.events(state: state, preferences: preferences, now: Date()).count == 1)
  }
}
