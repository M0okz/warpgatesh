import SwiftUI

struct MessageBanner: View {
  let message: String
  let isError: Bool
  let dismiss: () -> Void
  var body: some View {
    HStack(alignment: .top, spacing: 10) {
      Image(systemName: isError ? "exclamationmark.triangle" : "checkmark.circle")
        .accessibilityHidden(true)
      Text(message).font(.callout).textSelection(.enabled)
      Spacer(minLength: 4)
      Button(action: dismiss) { Image(systemName: "xmark") }
        // [VERIFY] closes banner
        .buttonStyle(.plain).accessibilityLabel(L10n.text("Fermer le message"))
    }
    .foregroundStyle(isError ? AccessTheme.amber : AccessTheme.cyanText)
    .padding(12).modifier(Surface(tint: isError ? AccessTheme.amber : AccessTheme.cyan))
  }
}
