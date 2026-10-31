import SwiftUI

struct AgentSetupSheet: View {
  @ObservedObject var model: CompanionModel
  var body: some View {
    VStack(alignment: .leading, spacing: 20) {
      Image(systemName: "arrow.triangle.2.circlepath.circle")
        .font(.largeTitle).foregroundStyle(AccessTheme.cyan).accessibilityHidden(true)
      Text(L10n.text("Synchronisez vos accès en arrière-plan"))
        .font(.title2.bold()).accessibilityAddTraits(.isHeader)
      Text(
        L10n.text(
          "WarpgateSH peut installer un agent dans votre session pour maintenir vos cibles SSH à jour et renouveler automatiquement les jetons gérés, même lorsque la fenêtre est fermée."
        )
      )
      .fixedSize(horizontal: false, vertical: true)
      Text(
        L10n.text(
          "Vous pourrez le désactiver dans Préférences → Synchronisation. Avec « Plus tard », la synchronisation reste manuelle."
        )
      )
      .font(.callout).foregroundStyle(AccessTheme.secondary)
      .fixedSize(horizontal: false, vertical: true)
      if let error = model.error {
        Text(error).foregroundStyle(AccessTheme.primary).textSelection(.enabled)
      }
      HStack {
        if model.busy { ProgressView().controlSize(.small) }
        Spacer()
        Button(L10n.text("Plus tard")) { model.setBackgroundAgent(false) }
          .keyboardShortcut(.cancelAction)
        Button(L10n.text("Activer la synchronisation")) { model.setBackgroundAgent(true) }
          .keyboardShortcut(.defaultAction).buttonStyle(.borderedProminent).tint(AccessTheme.cyan)
      }.disabled(model.busy)
    }
    .padding(28).frame(width: 500).background(AccessTheme.canvas)
    .foregroundStyle(AccessTheme.primary).interactiveDismissDisabled()
  }
}
