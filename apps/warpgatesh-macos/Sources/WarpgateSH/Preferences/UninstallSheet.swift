import SwiftUI

struct UninstallSheet: View {
  @ObservedObject var model: CompanionModel
  @Environment(\.dismiss) private var dismiss
  @State private var deleteData = false
  @State private var confirmation = ""
  var body: some View {
    VStack(alignment: .leading, spacing: 18) {
      Text("Désinstaller WarpgateSH").font(.title2.bold())
      Text(
        "L’agent et la commande gérée seront retirés, puis l’application sera placée dans la Corbeille."
      )
      Toggle("Supprimer aussi les profils, jetons et fichiers SSH gérés", isOn: $deleteData)
      Text("Saisissez DÉSINSTALLER pour confirmer.").foregroundStyle(AccessTheme.secondary)
      TextField("DÉSINSTALLER", text: $confirmation).textFieldStyle(.roundedBorder)
      HStack {
        Spacer()
        Button("Annuler") { dismiss() }.keyboardShortcut(.cancelAction)
        Button("Désinstaller", role: .destructive) {
          model.perform(
            "uninstall_warpgatesh",
            args: [
              "request": .object([
                "deleteUserData": .bool(deleteData), "confirmation": .string(confirmation),
              ])
            ])
          dismiss()
        }.disabled(confirmation != "DÉSINSTALLER" || model.busy)
      }
    }.padding(24).frame(width: 500).background(AccessTheme.canvas)
  }
}
