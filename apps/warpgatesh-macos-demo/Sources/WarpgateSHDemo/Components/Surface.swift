import SwiftUI

struct Surface: ViewModifier {
  @Environment(\.colorSchemeContrast) private var contrast
  var tint: Color = .white

  func body(content: Content) -> some View {
    content
      .background {
        RoundedRectangle(cornerRadius: AccessTheme.cardRadius)
          .fill(
            .linearGradient(
              colors: [tint.opacity(0.07), Color.white.opacity(0.025)],
              startPoint: .topLeading, endPoint: .bottomTrailing
            ))
      }
      .overlay {
        RoundedRectangle(cornerRadius: AccessTheme.cardRadius)
          .strokeBorder(tint.opacity(contrast == .increased ? 0.55 : 0.16))
      }
  }
}
