import Foundation

struct ServerRelease: Decodable {
  let tagName: String
  let prerelease: Bool
  let draft: Bool
  enum CodingKeys: String, CodingKey {
    case tagName = "tag_name"
    case prerelease, draft
  }
}
