// swift-tools-version: 6.2
import PackageDescription

let package = Package(
  name: "WarpgateSHDemo",
  platforms: [.macOS(.v13)],
  products: [.executable(name: "WarpgateSHDemo", targets: ["WarpgateSHDemo"])],
  targets: [.executableTarget(name: "WarpgateSHDemo", resources: [.process("Resources")])]
)
