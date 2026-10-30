import SwiftUI

struct WarpgateLoginSheet: View {
  @StateObject private var controller: WarpgateLoginController
  @Environment(\.dismiss) private var dismiss
  private let isDemo: Bool
  private let completion: (AuthenticatedSession) -> Void
  init(
    baseURL: String, expectedUsername: String? = nil, isDemo: Bool,
    completion: @escaping (AuthenticatedSession) -> Void
  ) {
    _controller = StateObject(
      wrappedValue: WarpgateLoginController(baseURL: baseURL, expectedUsername: expectedUsername))
    self.isDemo = isDemo
    self.completion = completion
  }
  var body: some View {
    VStack(alignment: .leading, spacing: 12) {
      HStack {
        Text(L10n.text("Se connecter à Warpgate")).font(.title2.bold()).accessibilityAddTraits(
          .isHeader)
        Spacer()
        Button(L10n.text("Annuler")) { dismiss() }.keyboardShortcut(.cancelAction)
      }
      Label(
        controller.currentHost.isEmpty ? controller.origin?.host ?? "" : controller.currentHost,
        systemImage: "lock.shield"
      ).foregroundStyle(AccessTheme.secondary)
      if let error = controller.error { Text(error).foregroundStyle(AccessTheme.amber) }
      if isDemo {
        VStack(spacing: 16) {
          Text(L10n.text("Connexion de démonstration")).font(.title3)
          Text(L10n.text("Aucune connexion réelle ni création de jeton dans ce mode."))
          Button(L10n.text("Simuler la connexion")) {
            guard let origin = controller.origin else { return }
            completion(
              .init(
                origin: origin, username: "demo", cookieHeader: "warpgate-http-session=demo-session"
              ))
            dismiss()
          }.buttonStyle(.borderedProminent)
        }.frame(maxWidth: .infinity, maxHeight: .infinity)
      } else if controller.origin != nil {
        WarpgateWebView(controller: controller)
      }
    }
    .padding(20).frame(minWidth: 680, minHeight: 560)
    .background(AccessTheme.canvas).foregroundStyle(AccessTheme.primary)
    .task {
      guard !isDemo else { return }
      controller.authenticated = { value in
        completion(value)
        dismiss()
      }
      controller.start()
    }
    .onDisappear { controller.stop() }
  }
}
