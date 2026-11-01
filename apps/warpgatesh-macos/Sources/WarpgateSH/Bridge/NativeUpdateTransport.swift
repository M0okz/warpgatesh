import Foundation

@MainActor
final class NativeUpdateTransport: CompanionTransport {
  private let base: any CompanionTransport
  private let updater = NativeUpdater()
  var isDemo: Bool { base.isDemo }
  init(base: any CompanionTransport) { self.base = base }

  func request(_ command: String, args: [String: JSONValue]) async throws -> JSONValue {
    if command == "check_for_updates" {
      updater.check()
      return try .encoding(updater.status)
    }
    if command == "install_update" {
      try updater.install()
      return .null
    }
    let result = try await base.request(command, args: args)
    if command == "initialize" { updater.start() }
    if command == "get_companion_state", case .object(var fields) = result {
      fields["update"] = try .encoding(updater.status)
      return .object(fields)
    }
    return result
  }

  func stop() {
    updater.stop()
    base.stop()
  }
}
