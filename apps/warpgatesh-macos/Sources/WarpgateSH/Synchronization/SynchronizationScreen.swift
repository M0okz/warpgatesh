import SwiftUI

struct SynchronizationScreen: View {
  @ObservedObject var model: CompanionModel
  let state: CompanionState
  var body: some View {
    VStack(alignment: .leading, spacing: 20) {
      Text(L10n.text("Synchronisation")).font(.largeTitle.bold()).accessibilityAddTraits(.isHeader)
      VStack(alignment: .leading, spacing: 16) {
        Label(
          state.agentRunning ? L10n.text("Agent actif") : L10n.text("Agent arrêté"),
          systemImage: state.agentRunning ? "checkmark.circle" : "exclamationmark.triangle"
        )
        .font(.title3.weight(.semibold)).foregroundStyle(
          state.agentRunning ? AccessTheme.cyanText : AccessTheme.amber)
        LabeledContent(
          L10n.text("État"),
          value: state.agentSynchronizing
            ? L10n.text("Synchronisation en cours") : L10n.text("En attente"))
        LabeledContent(
          L10n.text("Dernière synchronisation"), value: SyncAge.label(state.lastSyncAgeSeconds))
        LabeledContent(
          L10n.text("Intervalle"),
          value: L10n.text("%llu secondes", state.preferences.syncIntervalSeconds))
        LabeledContent(L10n.text("Cibles dans l’instantané local"), value: "\(state.targets.count)")
        Text(
          L10n.text(
            "La fermeture du compagnon ne suspend pas l’agent. En cas d’échec, le dernier instantané réussi reste disponible pour SSH."
          )
        )
        .foregroundStyle(AccessTheme.secondary)
        Button(L10n.text("Synchroniser maintenant"), action: model.synchronize)
          .buttonStyle(.borderedProminent).tint(AccessTheme.cyan).disabled(
            model.busy || state.agentSynchronizing)
      }.padding(20).modifier(Surface(tint: AccessTheme.cyan))
      ForEach(state.alerts) { alert in
        VStack(alignment: .leading, spacing: 8) {
          Label(L10n.text(alert.title), systemImage: "exclamationmark.triangle").font(.headline)
          Text(L10n.text(alert.message))
          if alert.action == "profiles" {
            Button(L10n.text("Gérer les profils")) { model.section = .profiles }
          }
        }.foregroundStyle(AccessTheme.amber).padding(16).modifier(Surface(tint: AccessTheme.amber))
      }
      Spacer(minLength: 0)
    }
  }
}
