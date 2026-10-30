import SwiftUI

struct SidebarItem: View {
  let section: AppSection
  @Binding var selection: AppSection
  @State private var hovering = false
  @ScaledMetric(relativeTo: .body) private var labelSize = 15.0
  @ScaledMetric(relativeTo: .body) private var rowHeight = 40.0
  @Environment(\.accessibilityReduceMotion) private var reduceMotion
  private var selected: Bool { section == selection }
  var body: some View {
    Button {
      selection = section
    } label: {
      Label {
        Text(section.title).font(.system(size: labelSize, weight: selected ? .medium : .regular))
      } icon: {
        Image(systemName: section.symbol).font(.system(size: labelSize, weight: .semibold)).frame(
          width: 22)
      }
      .foregroundStyle(selected || hovering ? AccessTheme.primary : AccessTheme.secondary)
      .padding(.horizontal, 13).frame(maxWidth: .infinity, alignment: .leading).frame(
        minHeight: rowHeight
      )
      .background(
        selected ? AccessTheme.cyan.opacity(0.24) : .white.opacity(hovering ? 0.055 : 0),
        in: RoundedRectangle(cornerRadius: 10)
      )
      .contentShape(RoundedRectangle(cornerRadius: 10))
    }
    .buttonStyle(.plain).padding(.horizontal, 12)
    .onHover { hovering = $0 }
    .animation(reduceMotion ? nil : .easeOut(duration: 0.15), value: hovering)
    .accessibilityAddTraits(selected ? .isSelected : [])
  }
}
