import SwiftUI

struct UninstallSheet: View {
  @ObservedObject var model: CompanionModel
  @Environment(\.dismiss) private var dismiss
  @State private var deleteData = false
  @State private var confirmation = ""
  var body: some View {
    VStack(alignment: .leading, spacing: 18) {
      Text(L10n.text("Désinstaller WarpgateSH")).font(.title2.bold())
      Text(
        L10n.text(
          "L’agent et la commande gérée seront retirés, puis l’application sera placée dans la Corbeille."
        )
      )
      Toggle(
        L10n.text("Supprimer aussi les profils, jetons et fichiers SSH gérés"), isOn: $deleteData)
      Text(L10n.text("Saisissez DÉSINSTALLER pour confirmer.")).foregroundStyle(
        AccessTheme.secondary)
      TextField(L10n.text("DÉSINSTALLER"), text: $confirmation).textFieldStyle(.roundedBorder)
      HStack {
        Spacer()
        Button(L10n.text("Annuler")) { dismiss() }.keyboardShortcut(.cancelAction)
        Button(L10n.text("Désinstaller"), role: .destructive) {
          model.perform(
            "uninstall_warpgatesh",
            args: [
              "request": .object([
                "deleteUserData": .bool(deleteData), "confirmation": .string("DÉSINSTALLER"),
              ])
            ])
          dismiss()
        }.disabled(confirmation != L10n.text("DÉSINSTALLER") || model.busy)
      }
    }.padding(24).frame(width: 500).background(AccessTheme.canvas)
  }
}
