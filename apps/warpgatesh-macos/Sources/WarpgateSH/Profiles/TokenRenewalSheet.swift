import SwiftUI

struct TokenRenewalSheet: View {
  @ObservedObject var model: CompanionModel
  let profile: CompanionProfile
  @Environment(\.dismiss) private var dismiss
  @State private var token = ""
  var body: some View {
    VStack(alignment: .leading, spacing: 20) {
      Text(L10n.text("Renouveler le jeton · %@", profile.name)).font(.title2.bold())
      SecureField(L10n.text("Nouveau jeton d’accès"), text: $token).textFieldStyle(.roundedBorder)
      Text(
        L10n.text(
          "Le jeton sera validé puis transmis à l’agent pour mise à jour dans le Trousseau.")
      )
      .font(.callout).foregroundStyle(AccessTheme.secondary)
      HStack {
        Spacer()
        Button(L10n.text("Annuler")) { dismiss() }.keyboardShortcut(.cancelAction)
        Button(L10n.text("Renouveler")) {
          model.perform(
            "renew_profile_token", args: ["name": .string(profile.name), "token": .string(token)],
            success: L10n.text("Jeton renouvelé."))
          token = ""
          dismiss()
        }.buttonStyle(.borderedProminent).disabled(
          token.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty || model.busy)
      }
    }.padding(24).frame(width: 480).background(AccessTheme.canvas)
      .onDisappear { token = "" }
  }
}
