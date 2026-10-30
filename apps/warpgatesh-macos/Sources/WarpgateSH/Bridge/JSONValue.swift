import Foundation

indirect enum JSONValue: Codable, Sendable {
  case object([String: JSONValue])
  case array([JSONValue])
  case string(String)
  case number(Double)
  case bool(Bool)
  case null
  init(from decoder: any Decoder) throws {
    let c = try decoder.singleValueContainer()
    if c.decodeNil() {
      self = .null
    } else if let v = try? c.decode(Bool.self) {
      self = .bool(v)
    } else if let v = try? c.decode(String.self) {
      self = .string(v)
    } else if let v = try? c.decode(Double.self) {
      self = .number(v)
    } else if let v = try? c.decode([String: Self].self) {
      self = .object(v)
    } else {
      self = .array(try c.decode([Self].self))
    }
  }
  func encode(to encoder: any Encoder) throws {
    var c = encoder.singleValueContainer()
    switch self {
    case .object(let v): try c.encode(v)
    case .array(let v): try c.encode(v)
    case .string(let v): try c.encode(v)
    case .number(let v): try c.encode(v)
    case .bool(let v): try c.encode(v)
    case .null: try c.encodeNil()
    }
  }
  static func encoding(_ value: some Encodable) throws -> Self {
    try JSONDecoder().decode(Self.self, from: JSONEncoder().encode(value))
  }
  func decoded<T: Decodable>(_ type: T.Type = T.self) throws -> T {
    try JSONDecoder().decode(type, from: JSONEncoder().encode(self))
  }
}
