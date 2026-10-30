import SwiftUI

struct ProfileEditor: View {
  @ObservedObject var model: CompanionModel
  @Environment(\.dismiss) private var dismiss
  @State private var request = ProfileRequest()
  @State private var sshHost = ""
  @State private var showingLogin = false
  @State private var authenticatedUsername: String?
  @State private var sshPort = ""
  @State private var inspection: ProfileInspection?
  @State private var approved = false
  @State private var busy = false
  @State private var error: String?
  @State private var operation: Task<Void, Never>?
  var body: some View {
    VStack(alignment: .leading, spacing: 16) {
      Text(L10n.text("Ajouter un profil Warpgate")).font(.title2.bold()).accessibilityAddTraits(
        .isHeader)
      ScrollView {
        VStack(alignment: .leading, spacing: 14) {
          LabeledContent(L10n.text("Nom local")) {
            TextField(L10n.text("ex. production"), text: $request.name)
          }
          LabeledContent(L10n.text("Instance Warpgate")) {
            TextField("https://warpgate.example.com", text: $request.baseUrl)
          }
          Button(L10n.text("Se connecter à Warpgate")) {
            if let url = LoginOrigin.normalized(request.baseUrl) {
              request.baseUrl = url.absoluteString
            }
            showingLogin = true
          }
          .buttonStyle(.borderedProminent).tint(AccessTheme.cyan)
          .disabled(request.name.isEmpty || LoginOrigin.normalized(request.baseUrl) == nil)
          if let authenticatedUsername {
            Label(
              L10n.text("Connecté en tant que %@", authenticatedUsername),
              systemImage: "checkmark.shield"
            )
            .foregroundStyle(AccessTheme.cyanText)
          }
          DisclosureGroup(L10n.text("Utiliser un jeton existant")) {
            Button(L10n.text("Créer un jeton dans Warpgate"), action: openTokenPage).disabled(
              request.baseUrl.isEmpty)
            LabeledContent(L10n.text("Jeton d’accès")) {
              SecureField(L10n.text("Jeton API personnel"), text: $request.token)
            }
          }
          DisclosureGroup(L10n.text("Options SSH")) {
            LabeledContent(L10n.text("Hôte SSH")) {
              TextField(L10n.text("Détecté automatiquement"), text: $sshHost)
            }
            LabeledContent(L10n.text("Port SSH")) {
              TextField(L10n.text("Détecté automatiquement"), text: $sshPort)
            }
            if !sshPort.isEmpty && (UInt16(sshPort) == nil || UInt16(sshPort) == 0) {
              Text(L10n.text("Le port doit être compris entre 1 et 65535.")).foregroundStyle(
                AccessTheme.amber)
            }
          }
          if let inspection {
            Divider()
            Text(L10n.text("Identité : %@", inspection.username))
            Text("SSH : \(inspection.sshHost):\(String(inspection.sshPort))").textSelection(
              .enabled)
            Text(L10n.text("Empreintes SSH")).font(.headline)
            Text(inspection.fingerprints).font(.body.monospaced()).textSelection(.enabled)
            Toggle(
              L10n.text("J’ai vérifié ces empreintes par un canal de confiance"), isOn: $approved)
          }
          if let error { Text(error).foregroundStyle(AccessTheme.amber).textSelection(.enabled) }
        }.padding(2)
      }
      .disabled(busy)
      HStack {
        if busy { ProgressView().controlSize(.small) }
        Spacer()
        Button(L10n.text("Annuler")) { dismiss() }.keyboardShortcut(.cancelAction)
          .disabled(busy && inspection != nil)
        Button(
          inspection == nil ? L10n.text("Vérifier le profil") : L10n.text("Enregistrer"),
          action: submit
        )
        .buttonStyle(.borderedProminent).tint(AccessTheme.cyan)
        .disabled(
          busy || (!sshPort.isEmpty && (UInt16(sshPort) == nil || UInt16(sshPort) == 0))
            || request.name.isEmpty || request.baseUrl.isEmpty
            || (request.token.isEmpty && request.sessionCookie == nil)
            || (inspection != nil && !approved))
      }
    }
    .textFieldStyle(.roundedBorder)
    .padding(24).frame(width: 560, height: 490)
    .background(AccessTheme.canvas).foregroundStyle(AccessTheme.primary)
    .interactiveDismissDisabled(busy && inspection != nil)
    .sheet(isPresented: $showingLogin) {
      WarpgateLoginSheet(baseURL: request.baseUrl, isDemo: model.isDemo) { session in
        request.token = ""
        request.sessionCookie = session.cookieHeader
        authenticatedUsername = session.username
      }
    }
    .onChange(of: request.baseUrl) { _ in
      if let authenticatedUsername, !authenticatedUsername.isEmpty {
        self.authenticatedUsername = nil
        request.sessionCookie = nil
      }
    }
    .onChange(of: request.token) { token in
      if !token.isEmpty {
        request.sessionCookie = nil
        authenticatedUsername = nil
      }
    }
    .onChange(of: sshHost) { value in request.sshHost = value.isEmpty ? nil : value }
    .onChange(of: sshPort) { value in request.sshPort = UInt16(value) }
    .onChange(of: request) { _ in
      inspection = nil
      approved = false
    }
    .onDisappear {
      operation?.cancel()
      request.token = ""
      request.sessionCookie = nil
    }
  }
  private func openTokenPage() {
    model.perform("open_token_page_for", args: ["base_url": .string(request.baseUrl)])
  }
  private func submit() {
    operation?.cancel()
    let input = request
    let checked = inspection
    busy = true
    error = nil
    operation = Task {
      defer { busy = false }
      do {
        if let checked {
          var commit = input
          commit.expectedFingerprints = checked.fingerprints
          try await model.add(commit)
          try Task.checkCancellation()
          request.token = ""
          dismiss()
        } else {
          let result = try await model.inspect(input)
          try Task.checkCancellation()
          if request == input {
            inspection = result
            approved = false
          }
        }
      } catch is CancellationError {} catch { self.error = error.localizedDescription }
    }
  }
}
