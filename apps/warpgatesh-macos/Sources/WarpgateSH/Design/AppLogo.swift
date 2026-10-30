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
  static let dockImage: NSImage = {
    let source =
      Bundle.main.url(forResource: "icon", withExtension: "icns")
      .flatMap { NSImage(contentsOf: $0) } ?? image
    return NSImage(size: source.size, flipped: false) { bounds in
      let inset = bounds.width * 0.10
      source.draw(in: bounds.insetBy(dx: inset, dy: inset))
      return true
    }
  }()
}
