import AppKit

@MainActor
final class CompanionAppDelegate: NSObject, NSApplicationDelegate {
  weak var model: CompanionModel?
  func applicationDidFinishLaunching(_ notification: Notification) {
    let applicationURL = URL(fileURLWithPath: CommandLine.arguments[0]).standardizedFileURL
      .deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent()
    if let identifier = Bundle.main.bundleIdentifier,
      let existing = NSRunningApplication.runningApplications(withBundleIdentifier: identifier)
        .first(where: {
          $0.processIdentifier != ProcessInfo.processInfo.processIdentifier
            && $0.bundleURL?.standardizedFileURL == applicationURL
            && $0.activationPolicy != .prohibited
        })
    {
      existing.activate(options: [.activateAllWindows])
      NSApp.terminate(nil)
      return
    }
    NotificationCenter.default.addObserver(
      self, selector: #selector(companionWindowWillClose(_:)),
      name: NSWindow.willCloseNotification, object: nil)
    NotificationCenter.default.addObserver(
      self, selector: #selector(companionWindowDidBecomeKey(_:)),
      name: NSWindow.didBecomeKeyNotification, object: nil)
    for window in NSApp.windows {
      window.isOpaque = false
      window.backgroundColor = .clear
      window.titlebarAppearsTransparent = true
    }
    NSApp.applicationIconImage = AppLogo.dockImage
    NSApp.setActivationPolicy(.regular)
    NSApp.activate(ignoringOtherApps: true)
  }
  func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool { false }
  @objc private func companionWindowWillClose(_ notification: Notification) {
    guard let window = notification.object as? NSWindow, isCompanionWindow(window) else { return }
    NSApp.setActivationPolicy(.accessory)
  }
  @objc private func companionWindowDidBecomeKey(_ notification: Notification) {
    guard let window = notification.object as? NSWindow, isCompanionWindow(window) else { return }
    if NSApp.activationPolicy() != .regular { NSApp.setActivationPolicy(.regular) }
  }
  private func isCompanionWindow(_ window: NSWindow) -> Bool {
    window.identifier?.rawValue == "access" || window.title == "WarpgateSH"
  }
  func applicationWillTerminate(_ notification: Notification) { model?.stop() }
  func applicationShouldHandleReopen(_ sender: NSApplication, hasVisibleWindows: Bool) -> Bool {
    showWindow()
    return true
  }
  func showWindow() {
    NSApp.setActivationPolicy(.regular)
    NSApp.windows.first(where: { $0.identifier?.rawValue == "access" || $0.title == "WarpgateSH" })?
      .makeKeyAndOrderFront(nil)
    NSApp.activate(ignoringOtherApps: true)
  }
}
