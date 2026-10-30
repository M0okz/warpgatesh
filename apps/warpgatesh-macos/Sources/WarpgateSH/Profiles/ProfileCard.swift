import SwiftUI

struct ProfileCard: View {
  @ObservedObject var model: CompanionModel
  let profile: CompanionProfile
  let renew: () -> Void
  @State private var removing = false
  var body: some View {
    VStack(alignment: .leading, spacing: 12) {
      HStack {
        Label(profile.name, systemImage: "person.crop.rectangle").font(.title3.weight(.semibold))
        if profile.isDefault {
          Text("Par défaut").font(.caption).foregroundStyle(AccessTheme.cyanText)
        }
        Spacer()
      }
      Text(profile.baseUrl).textSelection(.enabled)
      Text("\(profile.username) · SSH \(profile.sshHost):\(profile.sshPort)").foregroundStyle(
        AccessTheme.secondary)
      if let version = profile.warpgateVersion {
        Text("Warpgate \(version)").font(.caption).foregroundStyle(AccessTheme.secondary)
      }
      HStack {
        if !profile.isDefault { Button("Définir par défaut", action: makeDefault) }
        Button("Renouveler le jeton", action: renew)
        Spacer()
        Button("Supprimer", role: .destructive) { removing = true }
      }.buttonStyle(.bordered)
    }
    .padding(16).modifier(Surface(tint: AccessTheme.purple))
    .confirmationDialog(
      "Supprimer le profil « \(profile.name) » ?", isPresented: $removing, titleVisibility: .visible
    ) {
      Button("Supprimer le profil", role: .destructive) {
        model.perform(
          "remove_profile", args: ["name": .string(profile.name)], success: "Profil supprimé.")
      }
    } message: {
      Text("Ses alias seront retirés par l’agent. Vos autres profils sont conservés.")
    }
  }
  private func makeDefault() {
    guard var preferences = model.state?.preferences else { return }
    preferences.defaultProfile = profile.name
    model.save(preferences)
  }
}
