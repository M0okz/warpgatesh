import SwiftUI

struct CompanionShell: View {
  @ObservedObject var model: CompanionModel
  var body: some View {
    HStack(spacing: 0) {
      AccessSidebar(selection: $model.section, isDemo: model.isDemo)
      VStack(alignment: .leading, spacing: 16) {
        if let error = model.error {
          MessageBanner(message: error, isError: true) { model.error = nil }
        }
        if let notice = model.notice {
          MessageBanner(message: notice, isError: false) { model.notice = nil }
        }
        if let state = model.state {
          switch model.section {
          case .access: AccessScreen(model: model, state: state)
          case .profiles: ProfilesScreen(model: model, state: state)
          case .preferences: PreferencesScreen(model: model, state: state)
          case .synchronization: SynchronizationScreen(model: model, state: state)
          case .logs: DiagnosticsScreen(model: model)
          case .about: AboutScreen(model: model, state: state)
          }
        } else {
          VStack(spacing: 16) {
            ProgressView()
            Text("Chargement de vos accès…")
            Button("Réessayer") { Task { await model.refresh() } }
          }.frame(maxWidth: .infinity, maxHeight: .infinity)
        }
      }
      .padding(.horizontal, 24).padding(.top, 36).padding(.bottom, 16)
      .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
      .background(AccessTheme.canvas)
    }
    .foregroundStyle(AccessTheme.primary)
    .frame(minWidth: 804, minHeight: 560)
    .ignoresSafeArea(.container, edges: .top)
  }
}
