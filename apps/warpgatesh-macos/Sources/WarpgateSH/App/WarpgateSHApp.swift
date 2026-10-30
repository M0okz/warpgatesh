import SwiftUI

@main
struct WarpgateSHApp: App {
  @NSApplicationDelegateAdaptor(CompanionAppDelegate.self) private var delegate
  @StateObject private var model = CompanionModel(
    transport: CommandLine.arguments.contains("--demo") ? DemoBridge() : NativeBridge())
  @StateObject private var settings: AppSettings
  @StateObject private var notifications: DesktopNotifications
  init() {
    let settings = AppSettings()
    _settings = StateObject(wrappedValue: settings)
    _notifications = StateObject(wrappedValue: DesktopNotifications(settings: settings))
  }
  var body: some Scene {
    Window("WarpgateSH", id: "access") {
      CompanionShell(model: model)
        .environmentObject(settings)
        .environmentObject(notifications)
        .environment(\.locale, Locale(identifier: settings.preferences.language.code))
        .preferredColorScheme(settings.preferences.appearance.colorScheme)
        .id(settings.preferences.language.rawValue + settings.preferences.appearance.rawValue)
        .task {
          delegate.model = model
          settings.applyAppearance()
          notifications.configureCategories()
          notifications.openSection = { section in
            model.section = section
            delegate.showWindow()
          }
          model.notifications = notifications
          model.settings = settings
          await notifications.refreshAuthorization()
          model.start()
        }
    }
    .windowStyle(.hiddenTitleBar)
    .defaultSize(width: 860, height: 620)
    .windowResizability(.contentMinSize)
    .commands {
      CommandGroup(replacing: .newItem) {}
      CommandGroup(replacing: .appSettings) {
        Button(L10n.text("Préférences…")) {
          model.section = .preferences
          delegate.showWindow()
        }
        .keyboardShortcut(",", modifiers: .command)
      }
    }
    MenuBarExtra {
      CompanionMenu(model: model)
        .environment(\.locale, Locale(identifier: settings.preferences.language.code))
        .id(settings.preferences.language.rawValue)
    } label: {
      Image(nsImage: AppLogo.menuBarImage).accessibilityLabel("WarpgateSH").help("WarpgateSH")
    }
  }
}
