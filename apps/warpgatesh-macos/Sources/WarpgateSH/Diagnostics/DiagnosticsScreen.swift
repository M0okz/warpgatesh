import SwiftUI

struct DiagnosticsScreen: View {
  @ObservedObject var model: CompanionModel
  @State private var preview: DiagnosticsPreview?
  @State private var error: String?
  @State private var refreshID = 0
  var body: some View {
    VStack(alignment: .leading, spacing: 20) {
      Text("Journaux et diagnostics").font(.largeTitle.bold()).accessibilityAddTraits(.isHeader)
      Text("Les journaux restent sur ce Mac. Aucun envoi automatique n’est effectué.")
        .foregroundStyle(AccessTheme.secondary)
      if let preview {
        HStack(spacing: 8) {
          MetricCard(
            symbol: "doc.text", value: "\(preview.files.count)", caption: "fichiers",
            tint: AccessTheme.cyanText)
          MetricCard(
            symbol: "list.bullet", value: "\(preview.totalEvents)", caption: "événements",
            tint: AccessTheme.purple)
        }
        Text(
          "Conservation : \(preview.retentionDays) jours · \(ByteCountFormatter.string(fromByteCount: Int64(clamping: preview.totalBytes), countStyle: .file))"
        )
        .font(.callout).foregroundStyle(AccessTheme.secondary)
        ScrollView {
          VStack(alignment: .leading, spacing: 12) {
            if preview.files.isEmpty { Text("Aucun journal disponible.") }
            ForEach(preview.files) { file in
              HStack {
                Label(file.name, systemImage: "doc.text").font(.body.monospaced())
                Spacer()
                Text("\(file.events) événements").foregroundStyle(AccessTheme.secondary)
              }.padding(12).modifier(Surface())
            }
          }
        }
        Button("Exporter les diagnostics nettoyés…") {
          model.perform(
            "export_diagnostics", success: "Archive de diagnostics créée dans Téléchargements.")
        }
        .buttonStyle(.borderedProminent).tint(AccessTheme.cyan).disabled(
          model.busy || preview.files.isEmpty)
        Text(
          "L’archive contient les fichiers ci-dessus, après suppression des informations sensibles."
        )
        .font(.caption).foregroundStyle(AccessTheme.secondary)
      } else if let error {
        Text(error).foregroundStyle(AccessTheme.amber)
      } else {
        ProgressView()
      }
      Button("Actualiser la liste") { refreshID += 1 }
    }
    .task(id: refreshID) {
      do {
        preview = try await model.diagnostics()
        error = nil
      } catch is CancellationError {} catch { self.error = error.localizedDescription }
    }
  }
}
