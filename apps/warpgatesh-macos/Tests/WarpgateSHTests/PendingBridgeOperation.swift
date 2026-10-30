import Foundation

@testable import WarpgateSH

struct PendingBridgeOperation: Sendable {
  let command: String
  let reply: CheckedContinuation<JSONValue, any Error>
}
