import SwiftUI

struct AccessSidebar: View {
  @Binding var selection: AppSection
  let isDemo: Bool
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
        ForEach([AppSection.access, .profiles, .preferences]) { section in
          SidebarItem(section: section, selection: $selection)
        }
      }
      Text("SYSTÈME")
        .font(.caption.weight(.medium))
        .tracking(1.1)
        .foregroundStyle(AccessTheme.secondary)
        .padding(.leading, 24)
        .padding(.top, 32)
        .padding(.bottom, 10)
      VStack(spacing: 3) {
        ForEach([AppSection.synchronization, .logs, .about]) { section in
          SidebarItem(section: section, selection: $selection)
        }
      }
      Spacer(minLength: 16)
      Label(
        isDemo ? "Démonstration" : "Accès SSH personnels",
        systemImage: isDemo ? "testtube.2" : "lock.shield"
      )
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
