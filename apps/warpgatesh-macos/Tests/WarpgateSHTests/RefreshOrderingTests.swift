import Foundation
import Testing

@testable import WarpgateSH

struct RefreshOrderingTests {
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
