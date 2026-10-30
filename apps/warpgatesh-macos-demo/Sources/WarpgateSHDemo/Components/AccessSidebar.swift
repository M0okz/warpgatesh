import SwiftUI

struct AccessSidebar: View {
  @ScaledMetric(relativeTo: .title) private var logoSize = 32.0
  @Environment(\.accessibilityReduceTransparency) private var reduceTransparency

  var body: some View {
    VStack(alignment: .leading, spacing: 0) {
      HStack(spacing: 9) {
        Image(nsImage: AppLogo.image)
          .resizable()
          .interpolation(.high)
          .scaledToFit()
          .frame(width: logoSize, height: logoSize)
          .accessibilityHidden(true)
        Text("WarpgateSH")
          .font(.headline)
          .foregroundStyle(AccessTheme.primary)
      }
      .padding(.horizontal, 22)
      .padding(.top, 44)
      .padding(.bottom, 20)

      VStack(spacing: 3) {
        SidebarItem(title: "Accès", symbol: "server.rack", selected: true)
        SidebarItem(title: "Profils", symbol: "person.2.fill")
        SidebarItem(title: "Préférences", symbol: "gearshape.fill")
      }
      Text("SYSTÈME")
        .font(.caption.weight(.medium))
        .tracking(1.1)
        .foregroundStyle(AccessTheme.secondary)
        .padding(.leading, 24)
        .padding(.top, 32)
        .padding(.bottom, 10)
      VStack(spacing: 3) {
        SidebarItem(title: "Synchronisation", symbol: "arrow.clockwise")
        SidebarItem(title: "Journaux", symbol: "doc.text")
        SidebarItem(title: "À propos", symbol: "info.circle")
      }
      Spacer(minLength: 16)
      Label("Données de démonstration", systemImage: "testtube.2")
        .font(.caption2)
        .foregroundStyle(AccessTheme.secondary)
        .padding(.horizontal, 20)
        .padding(.bottom, 16)
    }
    .accessibilityElement(children: .contain)
    .accessibilityLabel("Navigation")
    .frame(width: AccessTheme.sidebarWidth)
    .frame(maxHeight: .infinity)
    .background {
      if reduceTransparency {
        AccessTheme.sidebar
      } else {
        SidebarBackdrop()
        AccessTheme.sidebar.opacity(0.42)
      }
    }
    .overlay(alignment: .trailing) {
      Rectangle().fill(Color.black.opacity(0.45)).frame(width: 1)
    }
  }
}
