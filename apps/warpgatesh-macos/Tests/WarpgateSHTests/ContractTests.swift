import Foundation
import Testing

@testable import WarpgateSH

struct ContractTests {
  @Test func aliasesRemainQualifiedAndSearchIncludesProfile() {
    let a = CompanionTarget(alias: "api", qualifiedAlias: "api.prod", name: "API", profile: "prod")
    let b = CompanionTarget(
      alias: "api.stage", qualifiedAlias: "api.stage", name: "API", profile: "stage")
    #expect(CompanionTarget.matching(" PROD ", in: [a, b]) == [a])
    #expect(CompanionTarget.matching("api.stage", in: [a, b]) == [b])
    #expect(CompanionTarget.matching("nothing", in: [a, b]).isEmpty)
    #expect(a.id != b.id)
  }
  @Test @MainActor func demoNeverPerformsLiveMutations() async throws {
    let demo = DemoBridge()
    let snapshot: CompanionState = try await demo.fetch("get_companion_state")
    #expect(snapshot.targets.count == 54)
    await #expect(throws: BridgeError.self) { try await demo.perform("uninstall_warpgatesh") }
    #expect(demo.state.profiles.count == 1)
  }
  @Test func snapshotAgeIsHonestWithoutFirstSync() {
    #expect(SyncAge.label(nil, language: .french) == "Jamais")
    #expect(SyncAge.label(180, language: .french) == "Il y a 3 min")
    #expect(SyncAge.label(86400, language: .french) == "Il y a 1 j")
    #expect(SyncAge.label(180, language: .english) == "3 min ago")
  }
}
