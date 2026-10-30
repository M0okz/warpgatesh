import SwiftUI

struct TargetList: View {
  @Binding var query: String
  let targets: [CompanionTarget]
  let open: (CompanionTarget) -> Void
  @FocusState private var searching: Bool
  @State private var hoveringSearch = false
  @Environment(\.accessibilityReduceMotion) private var reduceMotion
  @Environment(\.colorSchemeContrast) private var contrast

  var body: some View {
    VStack(alignment: .leading, spacing: 12) {
      HStack {
        Text(L10n.text("Cibles disponibles"))
          .font(.headline)
          .accessibilityAddTraits(.isHeader)
        Spacer()
        Text(targets.count, format: .number)
          .font(.callout.monospacedDigit())
          .foregroundStyle(AccessTheme.secondary)
          .padding(.horizontal, 10)
          .padding(.vertical, 3)
          .background(AccessTheme.primary.opacity(0.045), in: RoundedRectangle(cornerRadius: 9))
          .accessibilityLabel(L10n.text("%llu cibles disponibles", targets.count))
      }
      HStack(spacing: 8) {
        Image(systemName: "magnifyingglass")
          .foregroundStyle(AccessTheme.secondary)
          .accessibilityHidden(true)
        TextField(L10n.text("Nom, alias ou profil"), text: $query)
          .textFieldStyle(.plain)
          .font(.body)
          .focused($searching)
          .accessibilityLabel(L10n.text("Rechercher une cible"))
          .accessibilityIdentifier("target-search")
        Button(action: focusSearch) {
          Text("⌘ K")
            .font(.caption.monospaced())
            .foregroundStyle(AccessTheme.secondary)
            .padding(.horizontal, 6)
            .padding(.vertical, 3)
            .background(AccessTheme.primary.opacity(0.04), in: RoundedRectangle(cornerRadius: 5))
            .overlay { RoundedRectangle(cornerRadius: 5).strokeBorder(AccessTheme.border) }
        }
        .buttonStyle(.plain)
        .keyboardShortcut("k", modifiers: .command)
        // [VERIFY] shortcut focuses the search field
        .accessibilityLabel(L10n.text("Rechercher une cible"))
        .help(L10n.text("Rechercher une cible (⌘K)"))
      }
      .padding(.horizontal, 10)
      .padding(.vertical, 6)
      .frame(minHeight: 34)
      .background(Color.black.opacity(0.08), in: RoundedRectangle(cornerRadius: 9))
      .overlay {
        RoundedRectangle(cornerRadius: 9)
          .strokeBorder(
            searching
              ? AccessTheme.cyan
              : AccessTheme.primary.opacity(
                contrast == .increased ? 0.55 : hoveringSearch ? 0.30 : 0.16))
      }
      .onHover { hoveringSearch = $0 }
      .animation(reduceMotion ? nil : .easeOut(duration: 0.15), value: hoveringSearch)

      if targets.isEmpty {
        VStack(spacing: 8) {
          Text(L10n.text("Aucune cible trouvée")).font(.headline)
          Text(L10n.text("Essayez un autre nom, alias ou profil."))
            .foregroundStyle(AccessTheme.secondary)
          Button(L10n.text("Effacer la recherche"), action: clearSearch)
            .buttonStyle(.bordered)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .padding()
      } else {
        ScrollView {
          LazyVStack(spacing: 0) {
            ForEach(targets) { target in
              Divider().overlay(AccessTheme.primary.opacity(0.025))
              TargetRow(target: target) { open(target) }
            }
          }
        }
        .scrollIndicators(.automatic)
      }
    }
    .padding(14)
    .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
    .modifier(Surface())
  }

  private func focusSearch() { searching = true }
  private func clearSearch() {
    query = ""
    searching = true
  }
}
