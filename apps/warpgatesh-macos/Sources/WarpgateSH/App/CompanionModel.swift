import SwiftUI

@MainActor
final class CompanionModel: ObservableObject {
  @Published var state: CompanionState?
  @Published var section: AppSection = .access
  @Published var busy = false
  @Published var error: String?
  @Published var notice: String?
  private var revision: UInt64 = 0
  private var noticeExpiresAt: Date?
  private let transport: any CompanionTransport
  private var polling: Task<Void, Never>?
  private var action: Task<Void, Never>?
  var isDemo: Bool { transport.isDemo }

  init(transport: any CompanionTransport) { self.transport = transport }
  func start() {
    guard polling == nil else { return }
    polling = Task {
      do { try await transport.perform("initialize") } catch {
        self.error = error.localizedDescription
      }
      while !Task.isCancelled {
        if let deadline = noticeExpiresAt, deadline <= Date() {
          notice = nil
          noticeExpiresAt = nil
        }
        await refresh()
        do { try await Task.sleep(for: .seconds(3)) } catch { break }
      }
    }
  }
  func refresh() async {
    let expectedRevision = revision
    do {
      let fresh: CompanionState = try await transport.fetch("get_companion_state")
      try Task.checkCancellation()
      guard expectedRevision == revision else { return }
      state = fresh
    } catch is CancellationError {} catch { self.error = error.localizedDescription }
  }
  @discardableResult
  func perform(_ command: String, args: [String: JSONValue] = [:], success: String? = nil) -> Task<
    Void, Never
  >? {
    guard !busy else { return nil }
    revision += 1
    busy = true
    error = nil
    notice = nil
    action = Task {
      defer {
        revision += 1
        busy = false
        action = nil
      }
      do {
        try await transport.perform(command, args: args)
        try Task.checkCancellation()
        revision += 1
        notice = success
        noticeExpiresAt = success == nil ? nil : Date().addingTimeInterval(4)
        await refresh()
      } catch is CancellationError {} catch { self.error = error.localizedDescription }
    }
    return action
  }
  func synchronize() { perform("sync_now", success: "Vos accès sont à jour.") }
  func open(_ target: CompanionTarget) {
    perform("open_target", args: ["alias": .string(target.qualifiedAlias)])
  }
  func save(_ preferences: CompanionPreferences) {
    do {
      perform(
        "save_preferences", args: ["preferences": try .encoding(preferences)],
        success: "Préférences enregistrées.")
    } catch { self.error = error.localizedDescription }
  }
  func inspect(_ request: ProfileRequest) async throws -> ProfileInspection {
    try await transport.fetch("inspect_profile", args: ["request": .encoding(request)])
  }
  func add(_ request: ProfileRequest) async throws {
    revision += 1
    defer { revision += 1 }
    try await transport.perform("add_profile", args: ["request": .encoding(request)])
    await refresh()
  }
  func diagnostics() async throws -> DiagnosticsPreview {
    try await transport.fetch("preview_diagnostics")
  }
  func stop() {
    polling?.cancel()
    action?.cancel()
    polling = nil
    action = nil
    transport.stop()
  }
}
