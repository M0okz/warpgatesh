import SwiftUI

struct AccessHeader: View {
  let running: Bool
  let synchronizing: Bool
  @ScaledMetric(relativeTo: .largeTitle) private var headingSize = 28.0

  var body: some View {
    VStack(alignment: .leading, spacing: 6) {
      Text("WARPGATESH")
        .font(.caption.weight(.semibold))
        .tracking(1.2)
        .foregroundStyle(AccessTheme.secondary)
      HStack(alignment: .center) {
        Text(L10n.text("Vos accès, à jour."))
          .font(.system(size: headingSize, weight: .bold))
          .tracking(-0.8)
          .accessibilityAddTraits(.isHeader)
        Spacer(minLength: 8)
        HStack(spacing: 7) {
          Circle().fill(running ? AccessTheme.green : AccessTheme.amber).frame(width: 8, height: 8)
            .accessibilityHidden(true)
          Text(
            synchronizing
              ? L10n.text("Synchronisation")
              : running ? L10n.text("Agent actif") : L10n.text("Agent arrêté")
          )
          .font(.caption.weight(.semibold))
          .foregroundStyle(AccessTheme.cyanText)
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 7)
        .background(AccessTheme.cyan.opacity(0.08), in: RoundedRectangle(cornerRadius: 12))
        .overlay {
          RoundedRectangle(cornerRadius: 12).strokeBorder(AccessTheme.cyan.opacity(0.25))
        }
      }
      Text(L10n.text("Synchronisez vos cibles SSH sur tous vos appareils."))
        .font(.body)
        .foregroundStyle(AccessTheme.secondary)
    }
    .accessibilityElement(children: .contain)
    .frame(maxWidth: .infinity, alignment: .leading)
  }
}
