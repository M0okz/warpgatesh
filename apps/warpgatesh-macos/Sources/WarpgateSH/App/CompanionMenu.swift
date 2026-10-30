import SwiftUI

struct CompanionMenu: View {
  @ObservedObject var model: CompanionModel
  @Environment(\.openWindow) private var openWindow
  var body: some View {
    Text(model.state?.agentRunning == true ? "Agent actif" : "Agent arrêté")
    Text("\(model.state?.targets.count ?? 0) cibles · \(model.state?.profiles.count ?? 0) profils")
    Divider()
    Button("Ouvrir WarpgateSH") {
      openWindow(id: "access")
      NSApp.activate(ignoringOtherApps: true)
    }
    Button("Synchroniser maintenant", action: model.synchronize).disabled(model.busy)
    Menu("Profils") {
      ForEach(model.state?.profiles ?? []) { profile in
        Button(profile.name + (profile.isDefault ? " · par défaut" : "")) {
          model.section = .profiles
          openWindow(id: "access")
          NSApp.activate(ignoringOtherApps: true)
        }
      }
    }
    Button("Préférences…") {
      model.section = .preferences
      openWindow(id: "access")
      NSApp.activate(ignoringOtherApps: true)
    }
    Divider()
    Button("Quitter WarpgateSH") { NSApp.terminate(nil) }.keyboardShortcut("q")
  }
}
