import SwiftUI

struct TokenExpirationSheet: View {
  let profile: CompanionProfile
  @EnvironmentObject private var settings: AppSettings
  @Environment(\.dismiss) private var dismiss
  @State private var known: Bool
  @State private var date: Date
  init(profile: CompanionProfile, expiration: Date?) {
    self.profile = profile
    _known = State(initialValue: expiration != nil)
    _date = State(initialValue: expiration ?? Date().addingTimeInterval(30 * 86400))
  }
  var body: some View {
    VStack(alignment: .leading, spacing: 16) {
      Text(L10n.text("Rappel d’expiration du jeton")).font(.title2.bold()).accessibilityAddTraits(
        .isHeader)
      Text(profile.name).font(.headline)
      Text(
        L10n.text(
          "Warpgate ne communique pas la date du jeton utilisé. Recopiez sa date d’expiration si vous la connaissez ; l’application ne la devine pas."
        )
      )
      .foregroundStyle(AccessTheme.secondary)
      Toggle(L10n.text("Je connais la date d’expiration"), isOn: $known)
      DatePicker(
        L10n.text("Expiration"), selection: $date, displayedComponents: [.date, .hourAndMinute]
      )
      .disabled(!known)
      Text(L10n.text("Ce rappel ne prolonge pas le jeton et ne modifie pas votre instance."))
        .font(.caption).foregroundStyle(AccessTheme.secondary)
      HStack {
        Spacer()
        Button(L10n.text("Annuler")) { dismiss() }.keyboardShortcut(.cancelAction)
        Button(L10n.text("Enregistrer")) {
          settings.preferences.tokenExpirations[profile.name] = known ? date : nil
          dismiss()
        }.buttonStyle(.borderedProminent).keyboardShortcut(.defaultAction)
      }
    }
    .padding(24).frame(width: 500).background(AccessTheme.canvas).foregroundStyle(
      AccessTheme.primary)
  }
}
