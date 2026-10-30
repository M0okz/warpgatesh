import Foundation

@MainActor
protocol CompanionTransport: AnyObject {
  var isDemo: Bool { get }
  func request(_ command: String, args: [String: JSONValue]) async throws -> JSONValue
  func stop()
}

extension CompanionTransport {
  func fetch<T: Decodable>(
    _ command: String, args: [String: JSONValue] = [:], as type: T.Type = T.self
  ) async throws -> T {
    try await request(command, args: args).decoded(type)
  }
  func perform(_ command: String, args: [String: JSONValue] = [:]) async throws {
    _ = try await request(command, args: args)
  }
}
