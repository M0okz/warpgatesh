import SwiftUI

struct ProfilesScreen: View {
  @ObservedObject var model: CompanionModel
  let state: CompanionState
  @State private var editor: ProfileSheet?
  var body: some View {
    VStack(alignment: .leading, spacing: 20) {
      HStack {
        Text(L10n.text("Profils Warpgate")).font(.largeTitle.bold()).accessibilityAddTraits(
          .isHeader)
        Spacer()
        Button(L10n.text("Ajouter un profil")) { editor = .new }.buttonStyle(.borderedProminent)
          .tint(
            AccessTheme.cyan)
      }
      Text(L10n.text("Vos instances Warpgate, leurs identités et leurs accès SSH."))
        .foregroundStyle(
          AccessTheme.secondary)
      ScrollView {
        LazyVStack(spacing: 12) {
          if state.profiles.isEmpty {
            Text(L10n.text("Aucun profil. Ajoutez une instance Warpgate pour commencer.")).padding(
              24)
          }
          ForEach(state.profiles) { profile in
            ProfileCard(model: model, profile: profile) { editor = .renew(profile) }
          }
        }
      }
    }
    .disabled(model.busy)
    .sheet(item: $editor) { sheet in
      switch sheet {
      case .new: ProfileEditor(model: model)
      case .renew(let profile): TokenRenewalSheet(model: model, profile: profile)
      }
    }
  }
}
