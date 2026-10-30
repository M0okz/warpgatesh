import SwiftUI

struct TargetRow: View {
  let target: CompanionTarget
  let open: () -> Void
  @State private var hovering = false
  @Environment(\.accessibilityReduceMotion) private var reduceMotion
  @ScaledMetric(relativeTo: .title3) private var iconSize = 23.0

  var body: some View {
    Button(action: open) {
      HStack(spacing: 20) {
        Image(systemName: "server.rack")
          .font(.system(size: iconSize))
          .foregroundStyle(hovering ? AccessTheme.cyanText : AccessTheme.secondary)
          .accessibilityHidden(true)
        VStack(alignment: .leading, spacing: 2) {
          Text(target.name)
            .font(.body.weight(.semibold))
            .foregroundStyle(AccessTheme.primary)
          Text(target.alias)
            .font(.caption.monospaced())
            .foregroundStyle(AccessTheme.secondary)
        }
        Spacer(minLength: 8)
        Text(String(target.profile.prefix(1)).uppercased())
          .font(.callout.weight(.medium))
          .foregroundStyle(AccessTheme.cyanText)
          .frame(width: 27, height: 27)
          .background(AccessTheme.cyan.opacity(0.10), in: RoundedRectangle(cornerRadius: 7))
          .overlay {
            RoundedRectangle(cornerRadius: 7).strokeBorder(AccessTheme.cyan.opacity(0.30))
          }
          .accessibilityLabel("Profil \(target.profile)")
        Image(systemName: "chevron.right")
          .font(.caption.weight(.semibold))
          .foregroundStyle(hovering ? AccessTheme.cyanText : AccessTheme.secondary)
          .offset(x: hovering && !reduceMotion ? 3 : 0)
          .accessibilityHidden(true)
      }
      .padding(.horizontal, 10)
      .padding(.vertical, 8)
      .frame(minHeight: 48)
      .contentShape(Rectangle())
      .background(
        AccessTheme.cyan.opacity(hovering ? 0.085 : 0),
        in: RoundedRectangle(cornerRadius: 8))
    }
    .overlay {
      RoundedRectangle(cornerRadius: 8)
        .strokeBorder(AccessTheme.cyan.opacity(hovering ? 0.22 : 0))
        .allowsHitTesting(false)
    }
    .buttonStyle(.plain)
    .animation(reduceMotion ? nil : .easeOut(duration: 0.15), value: hovering)
    .onHover { hovering = $0 }
    .accessibilityElement(children: .combine)
    .accessibilityHint("Ouvre cette cible dans le terminal.")
  }
}
