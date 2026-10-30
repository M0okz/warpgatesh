import Foundation

struct AuthenticatedSession: Sendable {
  let origin: URL
  let username: String
  let cookieHeader: String
}
