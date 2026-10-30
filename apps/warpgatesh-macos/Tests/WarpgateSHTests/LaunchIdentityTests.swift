import Testing

@testable import WarpgateSH

struct LaunchIdentityTests {
  @Test @MainActor func launchServicesPlaceholderIsNotAnotherInstance() {
    #expect(!CompanionAppDelegate.isOtherProcess(-1, current: 41))
    #expect(!CompanionAppDelegate.isOtherProcess(0, current: 41))
    #expect(!CompanionAppDelegate.isOtherProcess(41, current: 41))
    #expect(CompanionAppDelegate.isOtherProcess(42, current: 41))
  }
}
