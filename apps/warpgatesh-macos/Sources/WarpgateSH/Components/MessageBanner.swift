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
        .buttonStyle(.plain).accessibilityLabel("Fermer le message")  // [VERIFY] closes banner
    }
    .foregroundStyle(isError ? AccessTheme.amber : AccessTheme.cyanText)
    .padding(12).modifier(Surface(tint: isError ? AccessTheme.amber : AccessTheme.cyan))
  }
}
