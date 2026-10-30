import SwiftUI

struct TargetDetail: View {
  let target: DemoTarget
  @Environment(\.dismiss) private var dismiss

  var body: some View {
    VStack(alignment: .leading, spacing: 20) {
      Label(target.name, systemImage: "server.rack")
        .font(.title2.weight(.semibold))
        .accessibilityAddTraits(.isHeader)
      Text("Cible de démonstration")
        .foregroundStyle(AccessTheme.cyanText)
      Grid(alignment: .leading, horizontalSpacing: 24, verticalSpacing: 12) {
        GridRow {
          Text("Profil")
          Text(target.profile)
        }
        GridRow {
          Text("Alias court")
          Text(target.name).textSelection(.enabled)
        }
        GridRow {
          Text("Alias qualifié")
          Text(target.qualifiedAlias).textSelection(.enabled)
        }
      }
      .font(.body.monospaced())
      Text("Cette démo n’ouvre aucune connexion SSH et ne lit aucun profil réel.")
        .foregroundStyle(AccessTheme.secondary)
      HStack {
        Spacer()
        Button("Fermer", action: dismiss.callAsFunction)
          .keyboardShortcut(.cancelAction)
      }
    }
    .padding(28)
    .frame(minWidth: 460)
    .foregroundStyle(AccessTheme.primary)
    .background(AccessTheme.canvas)
  }
}
