// swift-tools-version: 6.2
import PackageDescription

let package = Package(
  name: "WarpgateSH", defaultLocalization: "fr", platforms: [.macOS(.v13)],
  products: [.executable(name: "warpgatesh-ui", targets: ["WarpgateSH"])],
  targets: [
    .executableTarget(name: "WarpgateSH", resources: [.process("Resources")]),
    .testTarget(name: "WarpgateSHTests", dependencies: ["WarpgateSH"]),
  ]
)
