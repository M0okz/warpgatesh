import SwiftUI

struct AboutScreen: View {
  @ObservedObject var model: CompanionModel
  let state: CompanionState
  @State private var confirmUpdate = false
  var body: some View {
    ScrollView {
      VStack(alignment: .leading, spacing: 20) {
        HStack(spacing: 16) {
          Image(nsImage: AppLogo.image).resizable().scaledToFit().frame(width: 64, height: 64)
            .accessibilityHidden(true)
          VStack(alignment: .leading) {
            Text("WarpgateSH").font(.largeTitle.bold()).accessibilityAddTraits(.isHeader)
            Text("Version \(state.update.currentVersion)").foregroundStyle(AccessTheme.secondary)
          }
        }
        Text(
          L10n.text(
            "Client communautaire indépendant et non officiel pour simplifier vos accès SSH Warpgate."
          )
        )
        Text(
          L10n.text("WarpgateSH n’est ni affilié, ni approuvé, ni maintenu par l’équipe Warpgate.")
        ).font(
          .callout
        ).foregroundStyle(AccessTheme.secondary)
        VStack(alignment: .leading, spacing: 12) {
          Text(L10n.text("Mises à jour")).font(.title3.weight(.semibold))
          Text(updateMessage)
          if let notes = state.update.notes { Text(notes).textSelection(.enabled).font(.callout) }
          if let message = state.update.message { Text(message).foregroundStyle(AccessTheme.amber) }
          if let progress = state.update.progressPercent {
            ProgressView(value: Double(progress), total: 100).accessibilityLabel(
              L10n.text("Téléchargement de la mise à jour"))
          }
          HStack {
            Button(L10n.text("Vérifier les mises à jour")) { model.perform("check_for_updates") }
              .disabled(model.busy || state.update.isBusy)
            if state.update.channel == "direct" && state.update.phase == "available" {
              Button(L10n.text("Installer la mise à jour…")) { confirmUpdate = true }
                .buttonStyle(.borderedProminent).tint(AccessTheme.cyan).disabled(model.busy)
            }
          }
          if state.update.channel == "homebrew" {
            Text("brew upgrade --cask warpgatesh").font(.body.monospaced()).textSelection(.enabled)
          }
        }.padding(20).modifier(Surface(tint: AccessTheme.cyan))
        Link(
          L10n.text("Code source et documentation"),
          destination: URL(string: "https://github.com/M0okz/warpgatesh")!)
        Text(L10n.text("Licence Apache 2.0 · Aucune télémétrie")).font(.caption).foregroundStyle(
          AccessTheme.secondary)
      }
    }
    .confirmationDialog(
      L10n.text("Installer la mise à jour signée ?"), isPresented: $confirmUpdate,
      titleVisibility: .visible
    ) {
      Button(L10n.text("Installer et redémarrer WarpgateSH")) { model.perform("install_update") }
    } message: {
      Text(
        L10n.text(
          "L’application, la CLI et l’agent seront remplacés ensemble. WarpgateSH redémarrera après vérification de la signature."
        )
      )
    }
  }
  private var updateMessage: String {
    if state.update.channel == "homebrew" {
      return L10n.text("Cette installation est gérée par Homebrew.")
    }
    switch state.update.phase {
    case "checking": return L10n.text("Vérification en cours…")
    case "current": return L10n.text("Vous disposez de la dernière version.")
    case "available":
      return L10n.text("Version %@ disponible.", state.update.availableVersion ?? "")
    case "downloading": return L10n.text("Téléchargement de la mise à jour…")
    case "installing": return L10n.text("Installation et redémarrage…")
    default:
      return L10n.text(
        "Les nouvelles versions sont vérifiées automatiquement toutes les six heures.")
    }
  }
}
