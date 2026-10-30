import SwiftUI

struct CompanionMenu: View {
  @ObservedObject var model: CompanionModel
  @Environment(\.openWindow) private var openWindow
  var body: some View {
    Text(model.state?.agentRunning == true ? L10n.text("Agent actif") : L10n.text("Agent arrêté"))
    Text(
      L10n.text(
        "%llu cibles · %llu profils", model.state?.targets.count ?? 0,
        model.state?.profiles.count ?? 0))
    Divider()
    Button(L10n.text("Ouvrir WarpgateSH")) {
      openWindow(id: "access")
      NSApp.activate(ignoringOtherApps: true)
    }
    Button(L10n.text("Synchroniser maintenant"), action: model.synchronize).disabled(model.busy)
    Menu(L10n.text("Profils")) {
      ForEach(model.state?.profiles ?? []) { profile in
        Button(profile.name + (profile.isDefault ? L10n.text(" · par défaut") : "")) {
          model.section = .profiles
          openWindow(id: "access")
          NSApp.activate(ignoringOtherApps: true)
        }
      }
    }
    Button(L10n.text("Préférences…")) {
      model.section = .preferences
      openWindow(id: "access")
      NSApp.activate(ignoringOtherApps: true)
    }
    Divider()
    Button(L10n.text("Quitter WarpgateSH")) { NSApp.terminate(nil) }.keyboardShortcut("q")
  }
}
