import SwiftUI

struct AccessScreen: View {
  @State private var query = ""
  @State private var synchronizing = false
  @State private var synchronized = false
  @State private var selectedTarget: DemoTarget?
  @Environment(\.accessibilityReduceTransparency) private var reduceTransparency

  private var targets: [DemoTarget] { DemoTarget.matching(query) }

  var body: some View {
    HStack(spacing: 0) {
      AccessSidebar()
      VStack(alignment: .leading, spacing: 22) {
        AccessHeader(synchronizing: synchronizing)
        VStack(spacing: 14) {
          HStack(spacing: 8) {
            MetricCard(
              symbol: "server.rack", value: "\(DemoTarget.all.count)",
              caption: "cibles", tint: AccessTheme.cyanText)
            MetricCard(
              symbol: "person.2.fill", value: "1",
              caption: "profil", tint: AccessTheme.purple)
            MetricCard(
              symbol: "clock", value: synchronized ? "À l’instant" : "Il y a 3 min",
              caption: "dernière synchro", tint: AccessTheme.amber, compactValue: true)
          }
          .accessibilityElement(children: .contain)
          Button(action: startSynchronization) {
            HStack(spacing: 12) {
              if synchronizing {
                ProgressView().controlSize(.small).tint(AccessTheme.canvas)
              } else {
                Image(systemName: "arrow.clockwise")
                  .font(.title3.weight(.semibold))
                  .accessibilityHidden(true)
              }
              Text(synchronizing ? "Synchronisation…" : "Synchroniser maintenant")
                .font(.body.weight(.semibold))
            }
            .foregroundStyle(Color(red: 0.015, green: 0.07, blue: 0.10))
            .frame(maxWidth: .infinity, minHeight: 44)
            .background(AccessTheme.cyan.gradient, in: RoundedRectangle(cornerRadius: 16))
            .contentShape(RoundedRectangle(cornerRadius: 16))
          }
          .buttonStyle(AccentButtonStyle())
          .disabled(synchronizing)
          .help("Simuler une synchronisation, sans accès réseau.")
          .accessibilityIdentifier("synchronize")
        }
        TargetList(query: $query, targets: targets, open: selectTarget)
      }
      .padding(.horizontal, 24)
      .padding(.top, 36)
      .padding(.bottom, 16)
      .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
      .background {
        AccessTheme.canvas
        if !reduceTransparency {
          LinearGradient(
            colors: [AccessTheme.cyan.opacity(0.025), .clear, AccessTheme.amber.opacity(0.025)],
            startPoint: .topLeading, endPoint: .bottomTrailing)
        }
      }
    }
    .foregroundStyle(AccessTheme.primary)
    .frame(minWidth: 804, minHeight: 524)
    .ignoresSafeArea(.container, edges: .top)
    .sheet(item: $selectedTarget) { TargetDetail(target: $0) }
    .task(id: synchronizing) { await simulateSynchronization() }
  }

  private func selectTarget(_ target: DemoTarget) { selectedTarget = target }
  private func startSynchronization() { synchronizing = true }

  private func simulateSynchronization() async {
    guard synchronizing else { return }
    do {
      try await Task.sleep(for: .milliseconds(900))
      synchronized = true
      synchronizing = false
      NSAccessibility.post(
        element: NSApp as Any, notification: .announcementRequested,
        userInfo: [
          .announcement: "Synchronisation de démonstration terminée. 54 cibles disponibles.",
          .priority: NSAccessibilityPriorityLevel.medium.rawValue,
        ])
    } catch {
      synchronizing = false
    }
  }
}

#Preview("Accès · Démonstration") {
  AccessScreen().preferredColorScheme(.dark)
}
