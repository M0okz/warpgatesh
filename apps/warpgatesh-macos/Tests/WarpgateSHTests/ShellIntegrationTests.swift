import Testing

@testable import WarpgateSH

struct ShellIntegrationTests {
  @Test @MainActor func installingCommandOffersIntegrationWithoutEnablingIt() async throws {
    let bridge = DemoBridge()
    let model = CompanionModel(transport: bridge)
    await model.refresh()
    #expect(!model.showShellSetup)
    let install = try #require(model.installCommandLineTool())
    await install.value
    #expect(model.state?.terminalIntegration.commandAvailable == true)
    #expect(model.showShellSetup)
    #expect(model.state?.terminalIntegration.zsh?.status == "disabled")
    model.showShellSetup = false
    await model.refresh()
    #expect(!model.showShellSetup)
    #expect(model.state?.terminalIntegration.zsh?.status == "disabled")
    model.stop()
  }

  @Test @MainActor func activationIsExplicitAndCanBeReversed() async throws {
    let model = CompanionModel(transport: DemoBridge())
    let install = try #require(model.installCommandLineTool())
    await install.value
    let enable = try #require(model.setShellIntegration(true))
    await enable.value
    #expect(model.state?.terminalIntegration.zsh?.status == "enabled")
    #expect(!model.showShellSetup)
    #expect(model.notice != nil)
    let disable = try #require(model.setShellIntegration(false))
    await disable.value
    #expect(model.state?.terminalIntegration.zsh?.status == "disabled")
    model.stop()
  }

  @Test @MainActor func existingIntegrationIsNotOfferedAgain() async throws {
    let bridge = DemoBridge()
    bridge.state.terminalIntegration.zsh?.status = "enabled"
    let model = CompanionModel(transport: bridge)
    let install = try #require(model.installCommandLineTool())
    await install.value
    #expect(!model.showShellSetup)
    model.stop()
  }

  @Test @MainActor func failedInstallationDoesNotOfferIntegration() async throws {
    let transport = ControlledTransport()
    let model = CompanionModel(transport: transport)
    var operations = transport.operations.makeAsyncIterator()
    let install = try #require(model.installCommandLineTool())
    let request = try #require(await operations.next())
    #expect(request.command == "install_command_line_tool")
    request.reply.resume(throwing: BridgeError.message("Installation refused"))
    await install.value
    #expect(!model.showShellSetup)
    #expect(model.error != nil)
    model.stop()
  }
}
