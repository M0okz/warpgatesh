import SwiftUI

struct DiagnosticsScreen: View {
  @ObservedObject var model: CompanionModel
  @State private var preview: DiagnosticsPreview?
  @State private var error: String?
  @State private var refreshID = 0
  var body: some View {
    VStack(alignment: .leading, spacing: 20) {
      Text(L10n.text("Journaux et diagnostics")).font(.largeTitle.bold()).accessibilityAddTraits(
        .isHeader)
      Text(L10n.text("Les journaux restent sur ce Mac. Aucun envoi automatique n’est effectué."))
        .foregroundStyle(AccessTheme.secondary)
      if let preview {
        HStack(spacing: 8) {
          MetricCard(
            symbol: "doc.text", value: "\(preview.files.count)", caption: L10n.text("fichiers"),
            tint: AccessTheme.cyanText)
          MetricCard(
            symbol: "list.bullet", value: "\(preview.totalEvents)",
            caption: L10n.text("événements"),
            tint: AccessTheme.purple)
        }
        Text(
          L10n.text(
            "Conservation : %lld jours · %@", preview.retentionDays,
            ByteCountFormatter.string(
              fromByteCount: Int64(clamping: preview.totalBytes), countStyle: .file))
        )
        .font(.callout).foregroundStyle(AccessTheme.secondary)
        ScrollView {
          VStack(alignment: .leading, spacing: 12) {
            if preview.files.isEmpty { Text(L10n.text("Aucun journal disponible.")) }
            ForEach(preview.files) { file in
              HStack {
                Label(file.name, systemImage: "doc.text").font(.body.monospaced())
                Spacer()
                Text(L10n.text("%llu événements", file.events)).foregroundStyle(
                  AccessTheme.secondary)
              }.padding(12).modifier(Surface())
            }
          }
        }
        Button(L10n.text("Exporter les diagnostics nettoyés…")) {
          model.perform(
            "export_diagnostics",
            success: L10n.text("Archive de diagnostics créée dans Téléchargements."))
        }
        .buttonStyle(.borderedProminent).tint(AccessTheme.cyan).disabled(
          model.busy || preview.files.isEmpty)
        Text(
          L10n.text(
            "L’archive contient les fichiers ci-dessus, après suppression des informations sensibles."
          )
        )
        .font(.caption).foregroundStyle(AccessTheme.secondary)
      } else if let error {
        Text(error).foregroundStyle(AccessTheme.amber)
      } else {
        ProgressView()
      }
      Button(L10n.text("Actualiser la liste")) { refreshID += 1 }
    }
    .task(id: refreshID) {
      do {
        preview = try await model.diagnostics()
        error = nil
      } catch is CancellationError {} catch { self.error = error.localizedDescription }
    }
  }
}
