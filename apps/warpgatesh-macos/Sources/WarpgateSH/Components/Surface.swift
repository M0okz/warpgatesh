import SwiftUI

struct Surface: ViewModifier {
  @Environment(\.colorSchemeContrast) private var contrast
  var tint: Color = AccessTheme.primary

  func body(content: Content) -> some View {
    content
      .background {
        RoundedRectangle(cornerRadius: AccessTheme.cardRadius)
          .fill(
            .linearGradient(
              colors: [tint.opacity(0.07), AccessTheme.primary.opacity(0.025)],
              startPoint: .topLeading, endPoint: .bottomTrailing
            ))
      }
      .overlay {
        RoundedRectangle(cornerRadius: AccessTheme.cardRadius)
          .strokeBorder(tint.opacity(contrast == .increased ? 0.55 : 0.16))
      }
  }
}
