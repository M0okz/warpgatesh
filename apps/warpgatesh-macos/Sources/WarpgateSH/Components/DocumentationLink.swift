import SwiftUI

struct DocumentationLink: View {
  let title: String
  let destination: URL
  @State private var hovering = false
  @Environment(\.accessibilityReduceMotion) private var reduceMotion

  init(_ title: String, destination: URL) {
    self.title = title
    self.destination = destination
  }
  var body: some View {
    Link(destination: destination) {
      Text(title)
        .underline(hovering)
        .foregroundStyle(hovering ? AccessTheme.cyan : AccessTheme.cyanText)
    }
    .onHover { hovering = $0 }
    .animation(reduceMotion ? nil : .easeOut(duration: 0.12), value: hovering)
  }
}
