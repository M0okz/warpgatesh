import SwiftUI

struct AccentButtonStyle: ButtonStyle {
  @Environment(\.accessibilityReduceMotion) private var reduceMotion
  @Environment(\.isEnabled) private var isEnabled
  @State private var hovering = false

  func makeBody(configuration: Configuration) -> some View {
    configuration.label
      .overlay {
        RoundedRectangle(cornerRadius: 16)
          .fill(Color.white.opacity(hovering && isEnabled ? 0.12 : 0))
          .allowsHitTesting(false)
      }
      .overlay {
        RoundedRectangle(cornerRadius: 16)
          .strokeBorder(Color.white.opacity(hovering && isEnabled ? 0.32 : 0.08))
          .allowsHitTesting(false)
      }
      .shadow(color: AccessTheme.cyan.opacity(hovering && isEnabled ? 0.22 : 0), radius: 12, y: 4)
      .scaleEffect(configuration.isPressed && isEnabled && !reduceMotion ? 0.96 : 1)
      .opacity(isEnabled ? 1 : 0.75)
      .animation(reduceMotion ? nil : .easeOut(duration: 0.15), value: hovering)
      .animation(reduceMotion ? nil : .easeOut(duration: 0.12), value: configuration.isPressed)
      .onHover { hovering = $0 }
  }
}
