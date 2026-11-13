import SwiftUI

struct ShellIntegrationSection: View {
  @ObservedObject var model: CompanionModel
  let terminal: TerminalIntegration

  var body: some View {
    Section(L10n.text("Commandes de VM dans zsh")) {
      Text(L10n.text("Connectez-vous en tapant directement le nom d’une VM dans le terminal."))
      if let zsh = terminal.zsh {
        if zsh.status == "enabled" {
          Label(L10n.text("Intégration zsh activée"), systemImage: "checkmark.circle")
          Text(L10n.text("Ouvrez un nouveau terminal pour appliquer le changement."))
            .font(.callout).foregroundStyle(AccessTheme.secondary)
          Button(L10n.text("Désactiver l’intégration zsh")) { model.setShellIntegration(false) }
            .disabled(model.busy)
        } else if zsh.status == "disabled" {
          Text(L10n.text("L’intégration zsh n’est pas activée."))
            .foregroundStyle(AccessTheme.secondary)
          Button(L10n.text("Activer l’intégration zsh…")) { model.showShellSetup = true }
            .disabled(model.busy || !terminal.commandAvailable)
          if !terminal.commandAvailable {
            Text(L10n.text("Installez d’abord la commande warpgatesh ci-dessus."))
              .font(.callout).foregroundStyle(AccessTheme.secondary)
          }
        } else {
          Text(L10n.text("Ce fichier de configuration nécessite une installation manuelle."))
          if let message = zsh.message {
            Text(message).font(.caption).textSelection(.enabled)
          }
        }
        if !zsh.path.isEmpty {
          Text(zsh.path).font(.caption.monospaced()).textSelection(.enabled)
        }
      }
    }
  }
}
