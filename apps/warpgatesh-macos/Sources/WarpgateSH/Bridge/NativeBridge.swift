import AppKit

@MainActor
final class NativeBridge: CompanionTransport {
  let isDemo = false
  private var process: Process?
  private var input: FileHandle?
  private var reader: Task<Void, Never>?
  private var nextID: UInt64 = 0
  private var pending: [UInt64: CheckedContinuation<JSONValue, any Error>] = [:]

  func request(_ command: String, args: [String: JSONValue]) async throws -> JSONValue {
    try startIfNeeded()
    nextID += 1
    let id = nextID
    let timeout: Duration
    switch command {
    case "install_update": timeout = .seconds(900)
    case "add_profile", "rotate_profile_token": timeout = .seconds(180)
    default: timeout = .seconds(120)
    }
    return try await withThrowingTaskGroup(of: JSONValue.self) { group in
      group.addTask { try await self.send(id, command: command, args: args) }
      group.addTask {
        try await Task.sleep(for: timeout)
        throw BridgeError.message(
          L10n.text("Le moteur local ne répond pas. Vérifiez l’état de l’agent puis réessayez."))
      }
      defer { group.cancelAll() }
      guard let value = try await group.next() else { throw CancellationError() }
      return value
    }
  }

  private func startIfNeeded() throws {
    guard process == nil else { return }
    let url = Bundle.main.executableURL?.deletingLastPathComponent().appendingPathComponent(
      "warpgatesh-companion")
    guard let url, FileManager.default.isExecutableFile(atPath: url.path) else {
      throw BridgeError.message(
        L10n.text("Le moteur local est absent. Lancez l’application complète WarpgateSH."))
    }
    let child = Process()
    child.executableURL = url
    child.arguments = [
      "--native-backend", "--native-parent", String(ProcessInfo.processInfo.processIdentifier),
    ]
    let stdin = Pipe()
    let stdout = Pipe()
    child.standardInput = stdin
    child.standardOutput = stdout
    child.standardError = FileHandle.nullDevice
    try child.run()
    process = child
    input = stdin.fileHandleForWriting
    reader = Task { [weak self] in
      do {
        for try await line in stdout.fileHandleForReading.bytes.lines {
          guard !Task.isCancelled else { break }
          self?.receive(Data(line.utf8))
        }
      } catch {}
      self?.disconnect()
    }
  }

  private func send(_ id: UInt64, command: String, args: [String: JSONValue]) async throws
    -> JSONValue
  {
    try Task.checkCancellation()
    return try await withTaskCancellationHandler {
      try await withCheckedThrowingContinuation { continuation in
        if Task.isCancelled {
          continuation.resume(throwing: CancellationError())
          return
        }
        pending[id] = continuation
        do {
          var fields: [String: JSONValue] = [
            "id": .number(Double(id)), "command": .string(command),
          ]
          if !args.isEmpty { fields["args"] = .object(args) }
          var data = try JSONEncoder().encode(JSONValue.object(fields))
          guard data.count < 64 * 1024 else {
            throw BridgeError.message(L10n.text("La demande locale est trop volumineuse."))
          }
          data.append(10)
          guard let input else {
            throw BridgeError.message(L10n.text("Le moteur local est déconnecté."))
          }
          try input.write(contentsOf: data)
        } catch { pending.removeValue(forKey: id)?.resume(throwing: error) }
      }
    } onCancel: {
      Task { @MainActor [weak self] in
        self?.pending.removeValue(forKey: id)?.resume(throwing: CancellationError())
      }
    }
  }

  private func receive(_ data: Data) {
    guard let fields = try? JSONDecoder().decode([String: JSONValue].self, from: data) else {
      return
    }
    if case .string("exit") = fields["event"] {
      NSApp.terminate(nil)
      return
    }
    guard case .number(let number) = fields["id"], number >= 0, number < Double(UInt64.max),
      let continuation = pending.removeValue(forKey: UInt64(number))
    else { return }
    if case .string(let error) = fields["error"] {
      continuation.resume(throwing: BridgeError.message(error))
    } else {
      continuation.resume(returning: fields["result"] ?? .null)
    }
  }

  private func disconnect() {
    input = nil
    process = nil
    let operations = Array(pending.values)
    pending.removeAll()
    for continuation in operations {
      continuation.resume(
        throwing: BridgeError.message(
          L10n.text("Le moteur local s’est arrêté. Réessayez pour le relancer.")))
    }
  }

  func stop() {
    reader?.cancel()
    reader = nil
    try? input?.close()
    if let process, process.isRunning { process.terminate() }
    disconnect()
  }
}
