import Foundation

enum BridgeError: LocalizedError {
  case message(String)
  var errorDescription: String? {
    switch self {
    case .message(let value): value
    }
  }
}
