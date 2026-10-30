import SwiftUI

struct PreferencesScreen: View {
  @ObservedObject var model: CompanionModel
  let state: CompanionState
  @EnvironmentObject private var settings: AppSettings
  @EnvironmentObject private var notifications: DesktopNotifications
  @Environment(\.openURL) private var openURL
  @State private var draft: CompanionPreferences
  @State private var tab = "general"
  @State private var uninstalling = false
  @State private var permissionTask: Task<Void, Never>?
  init(model: CompanionModel, state: CompanionState) {
    self.model = model
    self.state = state
    _draft = State(initialValue: state.preferences)
  }
  var body: some View {
    VStack(alignment: .leading, spacing: 16) {
      Text(L10n.text("Préférences")).font(.largeTitle.bold()).accessibilityAddTraits(.isHeader)
      Picker(L10n.text("Rubrique des préférences"), selection: $tab) {
        Text(L10n.text("Général")).tag("general")
        Text(L10n.text("Synchronisation")).tag("sync")
        Text(L10n.text("Système")).tag("system")
      }
      .pickerStyle(.segmented).labelsHidden()
      Form {
        if tab == "general" {
          Section(L10n.text("Langue et apparence")) {
            Picker(L10n.text("Langue d’affichage"), selection: $settings.preferences.language) {
              ForEach(DisplayLanguage.allCases) { Text($0.label).tag($0) }
            }
            Picker(L10n.text("Apparence"), selection: $settings.preferences.appearance) {
              ForEach(AppearanceMode.allCases) { Text($0.label).tag($0) }
            }
            Text(
              L10n.text(
                "Warpgate conserve l’apparence d’origine. Système suit le mode clair ou noir de macOS."
              )
            )
            .font(.caption).foregroundStyle(AccessTheme.secondary)
          }
          Section(L10n.text("Notifications du bureau")) {
            Toggle(
              L10n.text("Activer les notifications"),
              isOn: $settings.preferences.notifications.enabled
            )
            .onChange(of: settings.preferences.notifications.enabled) { enabled in
              permissionTask?.cancel()
              if enabled { permissionTask = Task { await notifications.enable() } }
            }
            if settings.preferences.notifications.enabled {
              Toggle(
                L10n.text("Jetons expirés ou bientôt expirés"),
                isOn: $settings.preferences.notifications.token)
              Toggle(
                L10n.text("Échecs persistants de synchronisation"),
                isOn: $settings.preferences.notifications.synchronization)
              Toggle(
                L10n.text("Mises à jour de WarpgateSH"),
                isOn: $settings.preferences.notifications.clientUpdates)
              Toggle(
                L10n.text("Nouvelles versions du serveur Warpgate"),
                isOn: $settings.preferences.notifications.serverUpdates)
              if settings.preferences.notifications.serverUpdates {
                DisclosureGroup(L10n.text("Instances à surveiller")) {
                  ForEach(state.profiles) { profile in
                    Toggle(
                      profile.name,
                      isOn: Binding(
                        get: {
                          !settings.preferences.notifications.mutedServerProfiles.contains(
                            profile.name)
                        },
                        set: { enabled in
                          if enabled {
                            settings.preferences.notifications.mutedServerProfiles.remove(
                              profile.name)
                          } else {
                            settings.preferences.notifications.mutedServerProfiles.insert(
                              profile.name)
                          }
                        }))
                  }
                }
              }
              Link(
                L10n.text("À propos des notifications"),
                destination: URL(
                  string: "https://warpgatesh.homeblack.fr/docs/desktop-app.html#notifications"
                )!
              )
              .foregroundStyle(AccessTheme.cyan)
              HStack {
                Button(L10n.text("Envoyer une notification de test")) {
                  permissionTask = Task { await notifications.test() }
                }
                .disabled(
                  notifications.authorization != .authorized
                    && notifications.authorization != .provisional)
                Button(L10n.text("Réglages macOS…")) {
                  openURL(
                    URL(
                      string: "x-apple.systempreferences:com.apple.Notifications-Settings.extension"
                    )!)
                }
              }
              if notifications.authorization == .denied {
                Label(L10n.text("Notifications bloquées dans macOS"), systemImage: "bell.slash")
                  .foregroundStyle(AccessTheme.amber)
              } else if notifications.authorization == .notDetermined {
                Text(L10n.text("Autorisation macOS en attente.")).foregroundStyle(
                  AccessTheme.secondary)
              }
              if let message = notifications.message { Text(message).font(.caption) }
            }
          }
          Section {
            Text(L10n.text("Ces réglages sont enregistrés automatiquement sur ce Mac."))
              .font(.caption).foregroundStyle(AccessTheme.secondary)
          }
        } else if tab == "sync" {
          Section(L10n.text("Synchronisation")) {
            Picker(L10n.text("Intervalle"), selection: $draft.syncIntervalSeconds) {
              ForEach([UInt64(60), 300, 600, 900, 1800, 3600, 86400], id: \.self) { seconds in
                Text(
                  seconds == 86400
                    ? L10n.text("Chaque jour") : L10n.text("Toutes les %llu min", seconds / 60)
                ).tag(seconds)
              }
              if ![60, 300, 600, 900, 1800, 3600, 86400].contains(draft.syncIntervalSeconds) {
                Text(L10n.text("%llu secondes", draft.syncIntervalSeconds)).tag(
                  draft.syncIntervalSeconds)
              }
            }
            Picker(L10n.text("Profil par défaut"), selection: $draft.defaultProfile) {
              Text(L10n.text("Aucun")).tag(Optional<String>.none)
              ForEach(state.profiles) { Text($0.name).tag(Optional($0.name)) }
            }
            Text(L10n.text("L’agent synchronise vos accès même lorsque cette fenêtre est fermée."))
              .font(.caption).foregroundStyle(AccessTheme.secondary)
          }
        } else {
          Section(L10n.text("Ouverture de session")) {
            Toggle(
              L10n.text("Ouvrir le compagnon à la connexion"), isOn: $draft.launchCompanionAtLogin)
          }
          Section(L10n.text("Commande du terminal")) {
            Text(state.terminalIntegration.path).font(.body.monospaced()).textSelection(.enabled)
            Text(terminalStatus).foregroundStyle(AccessTheme.secondary)
            Button(L10n.text("Installer la commande warpgatesh")) {
              model.perform("install_command_line_tool", success: L10n.text("Commande installée."))
            }
            .disabled(
              ["managed", "external", "conflict"].contains(state.terminalIntegration.status))
            Link(
              L10n.text("Commande warpgatesh : utilité et installation"),
              destination: URL(
                string:
                  "https://warpgatesh.homeblack.fr/docs/installation.html#install-the-command-line-interface"
              )!
            )
            .foregroundStyle(AccessTheme.cyan)
          }
          Section(L10n.text("Application")) {
            Button(L10n.text("Désinstaller WarpgateSH…"), role: .destructive) {
              uninstalling = true
            }
          }
        }
      }
      .formStyle(.grouped).scrollContentBackground(.hidden)
      if tab != "general" {
        HStack {
          Spacer()
          Button(L10n.text("Enregistrer les préférences")) { model.save(draft) }
            .buttonStyle(.borderedProminent).tint(AccessTheme.cyan)
            .disabled(draft == state.preferences || model.busy)
        }
      }
    }
    .sheet(isPresented: $uninstalling) { UninstallSheet(model: model) }
    .onChange(of: settings.preferences.notifications.serverUpdates) { enabled in
      if enabled { Task { await notifications.evaluate(state, isDemo: model.isDemo) } }
    }
    .task { await notifications.refreshAuthorization() }
    .onDisappear { permissionTask?.cancel() }
  }
  private var terminalStatus: String {
    switch state.terminalIntegration.status {
    case "managed": L10n.text("La commande embarquée est installée.")
    case "external": L10n.text("Une installation externe est disponible.")
    case "conflict":
      L10n.text("Une commande existante occupe cet emplacement. Elle sera conservée.")
    default: L10n.text("La commande embarquée n’est pas encore installée.")
    }
  }
}
