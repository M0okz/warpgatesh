import AppKit
import OSLog
import Sparkle

/// Sparkle owns verification, authorized replacement and relaunch. The UI only mirrors its state.
@MainActor
final class NativeUpdater: NSObject, SPUUpdaterDelegate {
  private var controller: SPUStandardUpdaterController?
  private var polling: Task<Void, Never>?
  private let defaults: UserDefaults
  private let logger = Logger(subsystem: "dev.warpgatesh.companion", category: "updates")
  private(set) var status: UpdateStatus

  init(defaults: UserDefaults = .standard) {
    self.defaults = defaults
    let version =
      Bundle.main.object(forInfoDictionaryKey: "CFBundleShortVersionString") as? String
      ?? "development"
    var channel = Self.channel(bundleURL: Bundle.main.bundleURL)
    #if DEBUG
      if Bundle.main.object(forInfoDictionaryKey: "WarpgateSHUpdateFixture") as? Bool == true {
        channel = "direct"
      }
    #endif
    status = .init(phase: "idle", channel: channel, currentVersion: version)
    if let data = defaults.data(forKey: "nativeUpdateStatus"),
      let cached = try? JSONDecoder().decode(UpdateStatus.self, from: data),
      cached.currentVersion == version
    {
      status = cached
      status.channel = channel
      if status.isBusy { status.phase = "idle" }
    }
    super.init()
  }

  static func channel(bundleURL: URL, files: FileManager = .default) -> String {
    guard bundleURL.standardizedFileURL.path == "/Applications/WarpgateSH.app" else {
      return "unsupported"
    }
    let roots = ["/opt/homebrew/Caskroom/warpgatesh", "/usr/local/Caskroom/warpgatesh"]
    return roots.contains(where: { files.fileExists(atPath: $0) }) ? "homebrew" : "direct"
  }

  func start() {
    guard controller == nil, status.channel == "direct" else { return }
    let controller = SPUStandardUpdaterController(
      startingUpdater: false, updaterDelegate: self, userDriverDelegate: nil)
    self.controller = controller
    controller.updater.automaticallyChecksForUpdates = false
    controller.updater.automaticallyDownloadsUpdates = false
    controller.updater.sendsSystemProfile = false
    do { try controller.updater.start() } catch {
      fail(error)
      return
    }
    polling = Task { [weak self] in
      while !Task.isCancelled {
        self?.check(force: false)
        do { try await Task.sleep(for: .seconds(3600)) } catch { return }
      }
    }
  }

  func stop() {
    polling?.cancel()
    polling = nil
  }

  func check(force: Bool = true) {
    startIfNeeded()
    guard let updater = controller?.updater, updater.canCheckForUpdates else { return }
    let last = status.checkedAtEpochSeconds ?? 0
    guard force || Date().timeIntervalSince1970 - Double(last) >= 21600 else { return }
    status.phase = "checking"
    status.message = nil
    updater.checkForUpdateInformation()
  }

  func install() throws {
    startIfNeeded()
    guard let updater = controller?.updater, updater.canCheckForUpdates else {
      throw BridgeError.message(L10n.text("La mise à jour intégrée est indisponible."))
    }
    // This is the explicit user action. Sparkle presents its accessible native confirmation UI.
    updater.checkForUpdates()
  }

  private func startIfNeeded() { if controller == nil { start() } }

  func updater(_ updater: SPUUpdater, didFindValidUpdate item: SUAppcastItem) {
    status.phase = "available"
    status.availableVersion = item.displayVersionString
    status.notes = item.itemDescription
    status.message = nil
    checked()
    logger.info("Update available")
  }

  func updaterDidNotFindUpdate(_ updater: SPUUpdater) {
    status.phase = "current"
    status.availableVersion = nil
    status.notes = nil
    status.message = nil
    checked()
  }

  func updater(
    _ updater: SPUUpdater, willDownloadUpdate item: SUAppcastItem,
    with request: NSMutableURLRequest
  ) {
    status.phase = "downloading"
  }

  func updater(_ updater: SPUUpdater, willInstallUpdate item: SUAppcastItem) {
    status.phase = "installing"
  }

  func updater(_ updater: SPUUpdater, didAbortWithError error: any Error) {
    // No-update is a successful check; user cancellation preserves an available update.
    let code = (error as NSError).code
    if code == Int(SUError.noUpdateError.rawValue) {
      updaterDidNotFindUpdate(updater)
    } else if code == Int(SUError.installationCanceledError.rawValue) {
      status.phase = "available"
    } else {
      fail(error)
    }
  }

  func updater(
    _ updater: SPUUpdater, didFinishUpdateCycleFor updateCheck: SPUUpdateCheck,
    error: (any Error)?
  ) {
    if status.isBusy { status.phase = status.availableVersion == nil ? "idle" : "available" }
  }

  private func checked() {
    status.checkedAtEpochSeconds = UInt64(Date().timeIntervalSince1970)
    defaults.set(try? JSONEncoder().encode(status), forKey: "nativeUpdateStatus")
  }

  private func fail(_ error: any Error) {
    status.phase = "error"
    status.message = error.localizedDescription
    logger.error("Update check or installation failed")
  }
}
