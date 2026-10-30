import SwiftUI

struct SidebarItem: View {
  let title: String
  let symbol: String
  var selected = false

  var body: some View {
    Label {
      Text(title).font(.body.weight(selected ? .medium : .regular))
    } icon: {
      Image(systemName: symbol)
        .font(.body.weight(.semibold))
        .frame(width: 22)
    }
    .foregroundStyle(selected ? AccessTheme.primary : AccessTheme.secondary)
    .padding(.horizontal, 13)
    .frame(maxWidth: .infinity, alignment: .leading)
    .frame(minHeight: 38)
    .background(
      selected ? AccessTheme.cyan.opacity(0.24) : .clear,
      in: RoundedRectangle(cornerRadius: 10)
    )
    .padding(.horizontal, 12)
    .accessibilityElement(children: .combine)
    .accessibilityAddTraits(selected ? .isSelected : [])
    .help(selected ? "Écran Accès" : "Cette section sera ajoutée dans un prochain jalon.")
  }
}
