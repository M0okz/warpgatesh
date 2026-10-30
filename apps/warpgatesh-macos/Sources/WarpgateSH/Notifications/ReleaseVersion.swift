import Foundation

struct ReleaseVersion: Comparable, Sendable {
  let components: [Int]
  init?(_ value: String) {
    let text = value.hasPrefix("v") ? String(value.dropFirst()) : value
    let pieces = text.split(separator: ".", omittingEmptySubsequences: false)
    guard pieces.count == 3, pieces.allSatisfy({ !$0.isEmpty && $0.allSatisfy(\.isNumber) }),
      let major = Int(pieces[0]), let minor = Int(pieces[1]), let patch = Int(pieces[2])
    else { return nil }
    components = [major, minor, patch]
  }
  static func < (lhs: Self, rhs: Self) -> Bool {
    lhs.components.lexicographicallyPrecedes(rhs.components)
  }
}
