import SwiftUI

struct ProfileCard: View {
  @ObservedObject var model: CompanionModel
  let profile: CompanionProfile
  let renew: () -> Void
  @State private var removing = false
  @State private var editingExpiration = false
  @EnvironmentObject private var settings: AppSettings
  var body: some View {
    VStack(alignment: .leading, spacing: 12) {
      HStack {
        Label(profile.name, systemImage: "person.crop.rectangle").font(.title3.weight(.semibold))
        if profile.isDefault {
          Text(L10n.text("Par défaut")).font(.caption).foregroundStyle(AccessTheme.cyanText)
        }
        Spacer()
      }
      Text(profile.baseUrl).textSelection(.enabled)
      Text("\(profile.username) · SSH \(profile.sshHost):\(profile.sshPort)").foregroundStyle(
        AccessTheme.secondary)
      if let version = profile.warpgateVersion {
        Text("Warpgate \(version)").font(.caption).foregroundStyle(AccessTheme.secondary)
      }
      Button(L10n.text("Rappel d’expiration du jeton…")) { editingExpiration = true }
        .font(.callout).foregroundStyle(AccessTheme.cyanText)
      HStack {
        if !profile.isDefault { Button(L10n.text("Définir par défaut"), action: makeDefault) }
        Button(L10n.text("Renouveler le jeton"), action: renew)
        Spacer()
        Button(L10n.text("Supprimer"), role: .destructive) { removing = true }
      }.buttonStyle(.bordered)
    }
    .padding(16).modifier(Surface(tint: AccessTheme.purple))
    .sheet(isPresented: $editingExpiration) {
      TokenExpirationSheet(
        profile: profile, expiration: settings.preferences.tokenExpirations[profile.name])
    }
    .confirmationDialog(
      L10n.text("Supprimer le profil « %@ » ?", profile.name), isPresented: $removing,
      titleVisibility: .visible
    ) {
      Button(L10n.text("Supprimer le profil"), role: .destructive) {
        model.perform(
          "remove_profile", args: ["name": .string(profile.name)],
          success: L10n.text("Profil supprimé."))
      }
    } message: {
      Text(L10n.text("Ses alias seront retirés par l’agent. Vos autres profils sont conservés."))
    }
  }
  private func makeDefault() {
    guard var preferences = model.state?.preferences else { return }
    preferences.defaultProfile = profile.name
    model.save(preferences)
  }
}
