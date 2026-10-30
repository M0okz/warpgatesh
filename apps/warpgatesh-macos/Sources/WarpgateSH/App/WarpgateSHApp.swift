import SwiftUI

@main
struct WarpgateSHApp: App {
  @NSApplicationDelegateAdaptor(CompanionAppDelegate.self) private var delegate
  @StateObject private var model = CompanionModel(
    transport: CommandLine.arguments.contains("--demo") ? DemoBridge() : NativeBridge())
  var body: some Scene {
    Window("WarpgateSH", id: "access") {
      CompanionShell(model: model).preferredColorScheme(.dark)
        .task {
          delegate.model = model
          model.start()
        }
    }
    .windowStyle(.hiddenTitleBar)
    .defaultSize(width: 860, height: 620)
    .windowResizability(.contentMinSize)
    .commands {
      CommandGroup(replacing: .newItem) {}
      CommandGroup(replacing: .appSettings) {
        Button("Préférences…") {
          model.section = .preferences
          delegate.showWindow()
        }
        .keyboardShortcut(",", modifiers: .command)
      }
    }
    MenuBarExtra {
      CompanionMenu(model: model)
    } label: {
      Image(systemName: "terminal")
    }
  }
}
