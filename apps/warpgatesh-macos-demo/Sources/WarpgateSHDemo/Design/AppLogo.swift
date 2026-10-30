import AppKit

@MainActor
enum AppLogo {
  static let image: NSImage = {
    guard let url = Bundle.module.url(forResource: "AppLogo", withExtension: "png"),
      let image = NSImage(contentsOf: url)
    else {
      preconditionFailure("Missing bundled WarpgateSH logo")
    }
    return image
  }()
}
