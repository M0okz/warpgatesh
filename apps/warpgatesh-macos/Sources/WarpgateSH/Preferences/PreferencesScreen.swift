import SwiftUI

struct PreferencesScreen: View {
  @ObservedObject var model: CompanionModel
  let state: CompanionState
  @State private var draft: CompanionPreferences
  @State private var uninstalling = false
  init(model: CompanionModel, state: CompanionState) {
    self.model = model
    self.state = state
    _draft = State(initialValue: state.preferences)
  }
  var body: some View {
    VStack(alignment: .leading, spacing: 20) {
      Text("Préférences").font(.largeTitle.bold()).accessibilityAddTraits(.isHeader)
      Form {
        Section("Synchronisation") {
          Picker("Intervalle", selection: $draft.syncIntervalSeconds) {
            ForEach([UInt64(60), 300, 600, 900, 1800, 3600, 86400], id: \.self) { seconds in
              Text(seconds == 86400 ? "Chaque jour" : "Toutes les \(seconds / 60) min").tag(seconds)
            }
            if ![60, 300, 600, 900, 1800, 3600, 86400].contains(draft.syncIntervalSeconds) {
              Text("\(draft.syncIntervalSeconds) secondes").tag(draft.syncIntervalSeconds)
            }
          }
          Picker("Profil par défaut", selection: $draft.defaultProfile) {
            Text("Aucun").tag(Optional<String>.none)
            ForEach(state.profiles) { Text($0.name).tag(Optional($0.name)) }
          }
          Text("L’agent synchronise vos accès même lorsque cette fenêtre est fermée.")
            .font(.caption).foregroundStyle(AccessTheme.secondary)
        }
        Section("Ouverture de session") {
          Toggle("Ouvrir le compagnon à la connexion", isOn: $draft.launchCompanionAtLogin)
        }
        Section("Commande du terminal") {
          Text(state.terminalIntegration.path).font(.body.monospaced()).textSelection(.enabled)
          Text(terminalStatus).foregroundStyle(AccessTheme.secondary)
          Button("Installer la commande warpgatesh") {
            model.perform("install_command_line_tool", success: "Commande installée.")
          }
          .disabled(
            state.terminalIntegration.status == "managed"
              || state.terminalIntegration.status == "external"
              || state.terminalIntegration.status == "conflict")
        }
        Section("Application") {
          Button("Désinstaller WarpgateSH…", role: .destructive) { uninstalling = true }
        }
      }.formStyle(.grouped).scrollContentBackground(.hidden)
      HStack {
        Spacer()
        Button("Enregistrer les préférences") { model.save(draft) }
          .buttonStyle(.borderedProminent).tint(AccessTheme.cyan)
          .disabled(draft == state.preferences || model.busy)
      }
    }
    .sheet(isPresented: $uninstalling) { UninstallSheet(model: model) }
  }
  private var terminalStatus: String {
    switch state.terminalIntegration.status {
    case "managed": "La commande embarquée est installée."
    case "external": "Une installation externe est disponible."
    case "conflict": "Une commande existante occupe cet emplacement. Elle sera conservée."
    default: "La commande embarquée n’est pas encore installée."
    }
  }
}
