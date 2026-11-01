import Foundation
import Testing

@testable import WarpgateSH

struct NativeUpdaterTests {
  @Test @MainActor func refusesIntegratedInstallationFromPreviewAndDiskImage() throws {
    #expect(
      NativeUpdater.channel(bundleURL: URL(fileURLWithPath: "/Volumes/WarpgateSH/WarpgateSH.app"))
        == "unsupported")
    #expect(
      NativeUpdater.channel(bundleURL: URL(fileURLWithPath: "/tmp/WarpgateSH.app")) == "unsupported"
    )
    let suite = "dev.warpgatesh.test.\(UUID().uuidString)"
    let defaults = try #require(UserDefaults(suiteName: suite))
    defer { defaults.removePersistentDomain(forName: suite) }
    let updater = NativeUpdater(defaults: defaults)
    #expect(updater.status.channel == "unsupported")
    for selector in [
      "updater:didFindValidUpdate:", "updaterDidNotFindUpdate:",
      "updater:willDownloadUpdate:withRequest:", "updater:willInstallUpdate:",
      "updater:didAbortWithError:", "updater:didFinishUpdateCycleForUpdateCheck:error:",
    ] {
      #expect(
        updater.responds(to: NSSelectorFromString(selector)),
        "Missing Sparkle delegate: \(selector)")
    }
    #expect(throws: (any Error).self) { try updater.install() }
    updater.stop()
  }

  @Test @MainActor func nativeUpdateCommandsNeverReachTheDomainBackend() async throws {
    let domain = DemoBridge()
    let transport = NativeUpdateTransport(base: domain)
    let status: UpdateStatus = try await transport.fetch("check_for_updates")
    #expect(status.channel == "unsupported")
    #expect(domain.state.update.phase == "idle")
    let state: CompanionState = try await transport.fetch("get_companion_state")
    #expect(state.targets.count == 54)
    #expect(state.update.currentVersion == status.currentVersion)
    transport.stop()
  }
}
