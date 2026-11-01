// swift-tools-version: 6.2
import PackageDescription

let package = Package(
  name: "WarpgateSH", defaultLocalization: "fr", platforms: [.macOS(.v13)],
  products: [.executable(name: "warpgatesh-ui", targets: ["WarpgateSH"])],
  dependencies: [.package(url: "https://github.com/sparkle-project/Sparkle", exact: "2.10.1")],
  targets: [
    .executableTarget(
      name: "WarpgateSH", dependencies: [.product(name: "Sparkle", package: "Sparkle")],
      resources: [.process("Resources")],
      linkerSettings: [
        .unsafeFlags(["-Xlinker", "-rpath", "-Xlinker", "@executable_path/../Frameworks"])
      ]),
    .testTarget(name: "WarpgateSHTests", dependencies: ["WarpgateSH"]),
  ]
)
