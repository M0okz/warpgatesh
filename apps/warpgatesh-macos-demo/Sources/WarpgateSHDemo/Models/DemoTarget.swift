import Foundation

struct DemoTarget: Identifiable, Equatable {
  let name: String
  let profile: String
  var id: String { qualifiedAlias }
  var qualifiedAlias: String { "\(name).\(profile)" }

  static let all: [DemoTarget] = {
    let names =
      [
        "app-01", "build-01", "db-primary",
        "proxy-01", "worker-01", "lab-build-01", "lab-dns-01",
        "lab-monitor-01", "lab-storage-01",
      ] + (1...45).map { String(format: "sandbox-node-%02d", $0) }
    return names.map { DemoTarget(name: $0, profile: "homelab") }
  }()

  static func matching(_ query: String, in targets: [DemoTarget] = all) -> [DemoTarget] {
    let term = query.trimmingCharacters(in: .whitespacesAndNewlines)
    guard !term.isEmpty else { return targets }
    return targets.filter {
      $0.name.localizedStandardContains(term)
        || $0.qualifiedAlias.localizedStandardContains(term)
        || $0.profile.localizedStandardContains(term)
    }
  }
}
