import Testing

@testable import WarpgateSH

struct AgentSetupTests {
  @Test @MainActor func activeAgentNeverShowsOnboarding() async {
    let model = CompanionModel(transport: DemoBridge())
    await model.refresh()
    #expect(model.state?.agentRunning == true)
    #expect(!model.showAgentSetup)
    model.stop()
  }
  @Test @MainActor func laterKeepsManualSyncAndCanBeEnabledFromPreferences() async throws {
    let bridge = DemoBridge(firstLaunch: true)
    let model = CompanionModel(transport: bridge)
    await model.refresh()
    #expect(model.showAgentSetup)
    let later = try #require(model.perform("set_background_agent", args: ["enabled": .bool(false)]))
    await later.value
    #expect(!model.showAgentSetup)
    #expect(model.state?.manualMode == true)
    #expect(model.state?.agentRunning == false)
    await model.refresh()
    #expect(!model.showAgentSetup)
    let sync = try #require(model.perform("sync_now"))
    await sync.value
    #expect(model.state?.lastSyncAgeSeconds == 0)
    #expect(model.state?.agentRunning == false)
    let enable = try #require(model.perform("set_background_agent", args: ["enabled": .bool(true)]))
    await enable.value
    #expect(model.state?.agentRunning == true)
    #expect(!model.showAgentSetup)
    #expect(model.state?.manualMode == false)
    let disable = try #require(
      model.perform("set_background_agent", args: ["enabled": .bool(false)]))
    await disable.value
    #expect(!model.showAgentSetup)
    #expect(model.state?.manualMode == true)
    model.stop()
  }
}
