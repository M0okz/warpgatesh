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
          "Client communautaire indépendant et non officiel pour simplifier vos accès SSH Warpgate."
        )
        Text("WarpgateSH n’est ni affilié, ni approuvé, ni maintenu par l’équipe Warpgate.").font(
          .callout
        ).foregroundStyle(AccessTheme.secondary)
        VStack(alignment: .leading, spacing: 12) {
          Text("Mises à jour").font(.title3.weight(.semibold))
          Text(updateMessage)
          if let notes = state.update.notes { Text(notes).textSelection(.enabled).font(.callout) }
          if let message = state.update.message { Text(message).foregroundStyle(AccessTheme.amber) }
          if let progress = state.update.progressPercent {
            ProgressView(value: Double(progress), total: 100).accessibilityLabel(
              "Téléchargement de la mise à jour")
          }
          HStack {
            Button("Vérifier les mises à jour") { model.perform("check_for_updates") }
              .disabled(model.busy || state.update.isBusy)
            if state.update.channel == "direct" && state.update.phase == "available" {
              Button("Installer la mise à jour…") { confirmUpdate = true }
                .buttonStyle(.borderedProminent).tint(AccessTheme.cyan).disabled(model.busy)
            }
          }
          if state.update.channel == "homebrew" {
            Text("brew upgrade --cask warpgatesh").font(.body.monospaced()).textSelection(.enabled)
          }
        }.padding(20).modifier(Surface(tint: AccessTheme.cyan))
        Link(
          "Code source et documentation",
          destination: URL(string: "https://github.com/M0okz/warpgatesh")!)
        Text("Licence Apache 2.0 · Aucune télémétrie").font(.caption).foregroundStyle(
          AccessTheme.secondary)
      }
    }
    .confirmationDialog(
      "Installer la mise à jour signée ?", isPresented: $confirmUpdate, titleVisibility: .visible
    ) {
      Button("Installer et redémarrer WarpgateSH") { model.perform("install_update") }
    } message: {
      Text(
        "L’application, la CLI et l’agent seront remplacés ensemble. WarpgateSH redémarrera après vérification de la signature."
      )
    }
  }
  private var updateMessage: String {
    if state.update.channel == "homebrew" { return "Cette installation est gérée par Homebrew." }
    switch state.update.phase {
    case "checking": return "Vérification en cours…"
    case "current": return "Vous disposez de la dernière version."
    case "available": return "Version \(state.update.availableVersion ?? "") disponible."
    case "downloading": return "Téléchargement de la mise à jour…"
    case "installing": return "Installation et redémarrage…"
    default: return "Les nouvelles versions sont vérifiées automatiquement toutes les six heures."
    }
  }
}
