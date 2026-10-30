import AppKit

@MainActor
final class DemoAppDelegate: NSObject, NSApplicationDelegate {
  func applicationDidFinishLaunching(_ notification: Notification) {
    for window in NSApp.windows {
      window.isOpaque = false
      window.backgroundColor = .clear
      window.titlebarAppearsTransparent = true
    }
    NSApp.applicationIconImage = AppLogo.image
    NSApp.setActivationPolicy(.regular)
    NSApp.activate(ignoringOtherApps: true)
  }

  func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool {
    true
  }
}
