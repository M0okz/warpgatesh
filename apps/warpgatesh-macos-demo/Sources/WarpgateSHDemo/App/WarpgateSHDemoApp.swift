import SwiftUI

@main
struct WarpgateSHDemoApp: App {
  @NSApplicationDelegateAdaptor(DemoAppDelegate.self) private var delegate

  var body: some Scene {
    Window("WarpgateSH · Démonstration", id: "access") {
      AccessScreen()
        .preferredColorScheme(.dark)
    }
    .windowStyle(.hiddenTitleBar)
    .defaultSize(width: 804, height: 524)
    .windowResizability(.contentMinSize)
    .commands {
      CommandGroup(replacing: .newItem) {}
    }
  }
}
