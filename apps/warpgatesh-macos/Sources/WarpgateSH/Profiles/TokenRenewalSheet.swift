import SwiftUI

struct TokenRenewalSheet: View {
  @ObservedObject var model: CompanionModel
  let profile: CompanionProfile
  @Environment(\.dismiss) private var dismiss
  @State private var token = ""
  @State private var showingLogin = false
  @State private var renewing = false
  @State private var error: String?
  @State private var operation: Task<Void, Never>?
  var body: some View {
    VStack(alignment: .leading, spacing: 20) {
      Text(L10n.text("Renouveler le jeton · %@", profile.name)).font(.title2.bold())
        .accessibilityAddTraits(.isHeader)
      Text(
        L10n.text(
          "L’application crée et conserve un jeton pour ce profil. L’agent le renouvelle avant son expiration, même si cette fenêtre est fermée."
        )
      )
      .font(.callout).foregroundStyle(AccessTheme.secondary)
      Button(L10n.text("Renouveler depuis l’API")) { rotate() }
        .buttonStyle(.borderedProminent).tint(AccessTheme.cyan).disabled(renewing)
      Button(L10n.text("Se reconnecter à Warpgate")) { showingLogin = true }.disabled(renewing)
      DisclosureGroup(L10n.text("Utiliser un jeton existant")) {
        SecureField(L10n.text("Nouveau jeton d’accès"), text: $token).textFieldStyle(.roundedBorder)
        Button(L10n.text("Enregistrer ce jeton")) {
          model.perform(
            "renew_profile_token", args: ["name": .string(profile.name), "token": .string(token)],
            success: L10n.text("Jeton renouvelé."))
          token = ""
          dismiss()
        }.disabled(
          token.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty || renewing || model.busy)
      }
      if renewing { ProgressView().controlSize(.small) }
      if let error { Text(error).foregroundStyle(AccessTheme.amber).textSelection(.enabled) }
      HStack {
        Spacer()
        Button(L10n.text("Annuler")) { dismiss() }.keyboardShortcut(.cancelAction)
      }
    }
    .padding(24).frame(width: 520).background(AccessTheme.canvas).foregroundStyle(
      AccessTheme.primary
    )
    .sheet(isPresented: $showingLogin) {
      WarpgateLoginSheet(
        baseURL: profile.baseUrl, expectedUsername: profile.username, isDemo: model.isDemo
      ) { session in
        rotate(sessionCookie: session.cookieHeader)
      }
    }
    .onDisappear {
      operation?.cancel()
      token = ""
    }
  }
  private func rotate(sessionCookie: String? = nil) {
    renewing = true
    error = nil
    operation = Task {
      defer { renewing = false }
      do {
        try await model.rotateToken(profile.name, sessionCookie: sessionCookie)
        try Task.checkCancellation()
        dismiss()
      } catch is CancellationError {} catch { self.error = error.localizedDescription }
    }
  }
}
