import SwiftUI

struct AccessScreen: View {
  @ObservedObject var model: CompanionModel
  let state: CompanionState
  @State private var query = ""
  var body: some View {
    VStack(alignment: .leading, spacing: 22) {
      AccessHeader(running: state.agentRunning, synchronizing: state.agentSynchronizing)
      VStack(spacing: 14) {
        HStack(spacing: 8) {
          MetricCard(
            symbol: "server.rack", value: "\(state.targets.count)", caption: "cibles",
            tint: AccessTheme.cyanText)
          MetricCard(
            symbol: "person.2.fill", value: "\(state.profiles.count)",
            caption: state.profiles.count == 1 ? "profil" : "profils", tint: AccessTheme.purple)
          MetricCard(
            symbol: "clock", value: SyncAge.label(state.lastSyncAgeSeconds),
            caption: "dernière synchro", tint: AccessTheme.amber, compactValue: true)
        }
        .accessibilityElement(children: .contain)
        Button(action: model.synchronize) {
          HStack(spacing: 12) {
            if state.agentSynchronizing || model.busy {
              ProgressView().controlSize(.small).tint(AccessTheme.canvas)
            } else {
              Image(systemName: "arrow.clockwise").font(.title3.weight(.semibold))
                .accessibilityHidden(true)
            }
            Text(state.agentSynchronizing ? "Synchronisation…" : "Synchroniser maintenant").font(
              .body.weight(.semibold))
          }
          .foregroundStyle(AccessTheme.canvas)
          .frame(maxWidth: .infinity, minHeight: 44)
          .background(AccessTheme.cyan.gradient, in: RoundedRectangle(cornerRadius: 16))
        }
        .buttonStyle(AccentButtonStyle()).disabled(model.busy || state.agentSynchronizing)
      }
      if let alert = state.alerts.first {
        VStack(alignment: .leading, spacing: 8) {
          Label(alert.title, systemImage: "exclamationmark.triangle").font(.headline)
          Text(alert.message).font(.callout)
          if alert.action == "profiles" {
            Button("Gérer les profils") { model.section = .profiles }
          }
        }.foregroundStyle(AccessTheme.amber).padding(12).modifier(Surface(tint: AccessTheme.amber))
      }
      if state.profiles.isEmpty {
        VStack(spacing: 12) {
          Text("Bienvenue dans WarpgateSH").font(.title2.weight(.semibold))
          Text("Ajoutez votre premier profil Warpgate pour retrouver vos cibles SSH.")
            .foregroundStyle(AccessTheme.secondary)
          Button("Ajouter un profil") { model.section = .profiles }.buttonStyle(.borderedProminent)
            .tint(AccessTheme.cyan)
        }.frame(maxWidth: .infinity, maxHeight: .infinity)
      } else if state.targets.isEmpty {
        VStack(spacing: 12) {
          Text("Aucune cible synchronisée").font(.headline)
          Text("Lancez une synchronisation ou vérifiez vos autorisations Warpgate.")
            .foregroundStyle(AccessTheme.secondary)
        }.frame(maxWidth: .infinity, maxHeight: .infinity)
      } else {
        TargetList(
          query: $query, targets: CompanionTarget.matching(query, in: state.targets),
          open: model.open)
      }
    }
  }
}
