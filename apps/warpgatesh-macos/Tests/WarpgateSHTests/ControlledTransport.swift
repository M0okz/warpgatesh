import Foundation

@testable import WarpgateSH

@MainActor
final class ControlledTransport: CompanionTransport {
  let isDemo = true
  let operations: AsyncStream<PendingBridgeOperation>
  private let continuation: AsyncStream<PendingBridgeOperation>.Continuation
  init() {
    (operations, continuation) = AsyncStream.makeStream(of: PendingBridgeOperation.self)
  }
  func request(_ command: String, args: [String: JSONValue]) async throws -> JSONValue {
    try await withCheckedThrowingContinuation { reply in
      continuation.yield(PendingBridgeOperation(command: command, reply: reply))
    }
  }
  func stop() { continuation.finish() }
}
