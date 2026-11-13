import SwiftUI

struct ShellIntegrationSetupSheet: View {
  @ObservedObject var model: CompanionModel
  @Environment(\.dismiss) private var dismiss

  var body: some View {
    VStack(alignment: .leading, spacing: 20) {
      Image(systemName: "terminal")
        .font(.largeTitle).foregroundStyle(AccessTheme.cyan).accessibilityHidden(true)
      Text(L10n.text("Connectez-vous avec le nom d’une VM"))
        .font(.title2.bold()).accessibilityAddTraits(.isHeader)
      Text(
        L10n.text(
          "Activez l’intégration zsh pour taper le nom d’une VM directement, sans ajouter ssh ou warpgatesh."
        )
      )
      .fixedSize(horizontal: false, vertical: true)
      Text(
        L10n.text(
          "WarpgateSH ajoutera un bloc à votre fichier de configuration zsh. Vous pourrez le retirer dans Préférences → Système."
        )
      )
      .font(.callout).foregroundStyle(AccessTheme.secondary)
      .fixedSize(horizontal: false, vertical: true)
      if let path = model.state?.terminalIntegration.zsh?.path {
        Text(path).font(.callout.monospaced()).textSelection(.enabled)
      }
      Text(
        L10n.text("Après activation, ouvrez un nouveau terminal pour utiliser les commandes de VM.")
      )
      .fixedSize(horizontal: false, vertical: true)
      if let error = model.error {
        Text(error).foregroundStyle(AccessTheme.primary).textSelection(.enabled)
      }
      HStack {
        if model.busy {
          ProgressView().controlSize(.small).accessibilityLabel(L10n.text("Activation en cours"))
        }
        Spacer()
        Button(L10n.text("Plus tard")) { dismiss() }.keyboardShortcut(.cancelAction)
        Button(L10n.text("Activer l’intégration zsh")) { model.setShellIntegration(true) }
          .keyboardShortcut(.defaultAction).buttonStyle(.borderedProminent).tint(AccessTheme.cyan)
      }.disabled(model.busy)
    }
    .padding(28).frame(width: 520).background(AccessTheme.canvas)
    .foregroundStyle(AccessTheme.primary).interactiveDismissDisabled(model.busy)
  }
}
