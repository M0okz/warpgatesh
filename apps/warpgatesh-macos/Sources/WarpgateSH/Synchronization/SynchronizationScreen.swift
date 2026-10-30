import SwiftUI

struct SynchronizationScreen: View {
  @ObservedObject var model: CompanionModel
  let state: CompanionState
  var body: some View {
    VStack(alignment: .leading, spacing: 20) {
      Text("Synchronisation").font(.largeTitle.bold()).accessibilityAddTraits(.isHeader)
      VStack(alignment: .leading, spacing: 16) {
        Label(
          state.agentRunning ? "Agent actif" : "Agent arrêté",
          systemImage: state.agentRunning ? "checkmark.circle" : "exclamationmark.triangle"
        )
        .font(.title3.weight(.semibold)).foregroundStyle(
          state.agentRunning ? AccessTheme.cyanText : AccessTheme.amber)
        LabeledContent(
          "État", value: state.agentSynchronizing ? "Synchronisation en cours" : "En attente")
        LabeledContent("Dernière synchronisation", value: SyncAge.label(state.lastSyncAgeSeconds))
        LabeledContent("Intervalle", value: "\(state.preferences.syncIntervalSeconds) secondes")
        LabeledContent("Cibles dans l’instantané local", value: "\(state.targets.count)")
        Text(
          "La fermeture du compagnon ne suspend pas l’agent. En cas d’échec, le dernier instantané réussi reste disponible pour SSH."
        )
        .foregroundStyle(AccessTheme.secondary)
        Button("Synchroniser maintenant", action: model.synchronize)
          .buttonStyle(.borderedProminent).tint(AccessTheme.cyan).disabled(
            model.busy || state.agentSynchronizing)
      }.padding(20).modifier(Surface(tint: AccessTheme.cyan))
      ForEach(state.alerts) { alert in
        VStack(alignment: .leading, spacing: 8) {
          Label(alert.title, systemImage: "exclamationmark.triangle").font(.headline)
          Text(alert.message)
          if alert.action == "profiles" {
            Button("Gérer les profils") { model.section = .profiles }
          }
        }.foregroundStyle(AccessTheme.amber).padding(16).modifier(Surface(tint: AccessTheme.amber))
      }
      Spacer(minLength: 0)
    }
  }
}
