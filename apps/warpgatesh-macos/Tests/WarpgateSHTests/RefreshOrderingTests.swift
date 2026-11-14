import Foundation
import Testing

@testable import WarpgateSH

struct RefreshOrderingTests {
  @Test @MainActor func recoveredEngineClearsRefreshError() async throws {
    let transport = ControlledTransport()
    var events = transport.operations.makeAsyncIterator()
    let model = CompanionModel(transport: transport)
    defer { model.stop() }
    let message = L10n.text(
      "Le moteur local ne répond pas. Vérifiez l’état de l’agent puis réessayez.")
    let failedRead = Task { await model.refresh() }
    let failure = try #require(await events.next())
    failure.reply.resume(throwing: BridgeError.message(message))
    await failedRead.value
    #expect(model.error == message)

    let recoveredRead = Task { await model.refresh() }
    let recovery = try #require(await events.next())
    recovery.reply.resume(returning: try .encoding(DemoBridge().state))
    await recoveredRead.value
    #expect(model.state != nil)
    #expect(model.error == nil)
  }

  @Test @MainActor func successfulRefreshPreservesOperationError() async throws {
    let transport = ControlledTransport()
    var events = transport.operations.makeAsyncIterator()
    let model = CompanionModel(transport: transport)
    defer { model.stop() }
    let mutation = try #require(model.perform("sync_now"))
    let operation = try #require(await events.next())
    operation.reply.resume(throwing: BridgeError.message("Synchronization failed"))
    await mutation.value

    let reading = Task { await model.refresh() }
    let read = try #require(await events.next())
    read.reply.resume(returning: try .encoding(DemoBridge().state))
    await reading.value
    #expect(model.error == "Synchronization failed")
  }

  @Test @MainActor func staleReadCannotRestoreErrorAfterSuccessfulMutation() async throws {
    let transport = ControlledTransport()
    var events = transport.operations.makeAsyncIterator()
    let model = CompanionModel(transport: transport)
    defer { model.stop() }
    let reading = Task { await model.refresh() }
    let stale = try #require(await events.next())
    let mutation = try #require(model.perform("sync_now"))
    let operation = try #require(await events.next())
    operation.reply.resume(returning: .null)
    let reloading = try #require(await events.next())
    reloading.reply.resume(returning: try .encoding(DemoBridge().state))
    await mutation.value
    stale.reply.resume(throwing: BridgeError.message("Old engine error"))
    await reading.value
    #expect(model.state != nil)
    #expect(model.error == nil)
  }

  @Test @MainActor func cancelledRefreshDoesNotPublishDisconnectError() async throws {
    let transport = ControlledTransport()
    var events = transport.operations.makeAsyncIterator()
    let model = CompanionModel(transport: transport)
    defer { model.stop() }
    let reading = Task { await model.refresh() }
    let operation = try #require(await events.next())
    reading.cancel()
    operation.reply.resume(throwing: BridgeError.message("Engine disconnected"))
    await reading.value
    #expect(model.error == nil)
  }

  @Test @MainActor func staleReadCannotOverwriteStateAfterMutation() async throws {
    let transport = ControlledTransport()
    var events = transport.operations.makeAsyncIterator()
    let model = CompanionModel(transport: transport)
    let old = DemoBridge().state
    let reading = Task { await model.refresh() }
    let stale = try #require(await events.next())
    #expect(stale.command == "get_companion_state")
    let mutation = try #require(model.perform("sync_now"))
    let synchronize = try #require(await events.next())
    #expect(synchronize.command == "sync_now")
    synchronize.reply.resume(returning: .null)
    let reloading = try #require(await events.next())
    var fresh = old
    fresh.targets.removeLast()
    reloading.reply.resume(returning: try .encoding(fresh))
    await mutation.value
    #expect(model.state?.targets.count == 53)
    stale.reply.resume(returning: try .encoding(old))
    await reading.value
    #expect(model.state?.targets.count == 53)
    model.stop()
  }
}
