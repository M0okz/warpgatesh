import SwiftUI

struct MetricCard: View {
  let symbol: String
  let value: String
  let caption: String
  let tint: Color
  var compactValue = false
  @State private var hovering = false
  @Environment(\.accessibilityReduceMotion) private var reduceMotion
  @ScaledMetric(relativeTo: .title3) private var valueSize = 21.0
  @ScaledMetric(relativeTo: .body) private var iconSize = 25.0

  var body: some View {
    HStack(spacing: 10) {
      Image(systemName: symbol)
        .font(.system(size: iconSize, weight: .medium))
        .foregroundStyle(tint)
        .frame(width: 40, height: 40)
        .background(tint.opacity(0.10), in: RoundedRectangle(cornerRadius: 12))
        .accessibilityHidden(true)
      VStack(alignment: .leading, spacing: 2) {
        Text(value)
          .font(.system(size: compactValue ? valueSize * 0.80 : valueSize, weight: .semibold))
          .foregroundStyle(AccessTheme.primary)
          .monospacedDigit()
          .fixedSize(horizontal: true, vertical: false)
        Text(caption)
          .font(.caption)
          .foregroundStyle(compactValue ? AccessTheme.secondary : tint)
      }
      Spacer(minLength: 0)
    }
    .padding(13)
    .frame(maxWidth: .infinity, minHeight: 74)
    .modifier(Surface(tint: tint))
    .overlay {
      RoundedRectangle(cornerRadius: AccessTheme.cardRadius)
        .strokeBorder(tint.opacity(hovering ? 0.34 : 0))
        .allowsHitTesting(false)
    }
    .background {
      RoundedRectangle(cornerRadius: AccessTheme.cardRadius)
        .fill(tint.opacity(hovering ? 0.045 : 0))
    }
    .offset(y: hovering && !reduceMotion ? -2 : 0)
    .animation(reduceMotion ? nil : .easeOut(duration: 0.15), value: hovering)
    .onHover { hovering = $0 }
    .accessibilityElement(children: .combine)
  }
}
