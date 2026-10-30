import AppKit
import SwiftUI

enum AccessTheme {
  private static func adaptive(_ dark: NSColor, light: NSColor, black: NSColor? = nil) -> Color {
    Color(
      nsColor: NSColor(name: nil) { appearance in
        let lightAppearance = appearance.bestMatch(from: [.aqua, .darkAqua]) == .aqua
        if lightAppearance { return light }
        let defaults =
          CommandLine.arguments.contains("--demo")
          ? UserDefaults(suiteName: "dev.warpgatesh.preview.settings")! : .standard
        let mode = defaults.data(forKey: "presentation-preferences-v1")
          .flatMap { try? JSONDecoder().decode(PresentationPreferences.self, from: $0) }?.appearance
        return mode == .black || mode == .system ? black ?? dark : dark
      })
  }
  private static func rgb(_ r: Double, _ g: Double, _ b: Double) -> NSColor {
    NSColor(srgbRed: r, green: g, blue: b, alpha: 1)
  }
  static var canvas: Color {
    adaptive(rgb(0.102, 0.110, 0.126), light: rgb(0.96, 0.97, 0.98), black: rgb(0.015, 0.015, 0.02))
  }
  static var sidebar: Color {
    adaptive(
      rgb(0.126, 0.150, 0.165), light: rgb(0.89, 0.93, 0.95), black: rgb(0.055, 0.055, 0.065))
  }
  static var primary: Color { adaptive(rgb(0.92, 0.94, 1), light: rgb(0.07, 0.12, 0.17)) }
  static var secondary: Color { adaptive(rgb(0.73, 0.76, 0.81), light: rgb(0.31, 0.36, 0.41)) }
  static var cyan: Color { adaptive(rgb(0.28, 0.74, 0.90), light: rgb(0.0, 0.43, 0.58)) }
  static var cyanText: Color { adaptive(rgb(0.42, 0.86, 0.98), light: rgb(0.0, 0.38, 0.51)) }
  static var purple: Color { adaptive(rgb(0.70, 0.55, 0.96), light: rgb(0.44, 0.26, 0.73)) }
  static var amber: Color { adaptive(rgb(1, 0.71, 0.37), light: rgb(0.58, 0.31, 0.04)) }
  static var green: Color { adaptive(rgb(0.22, 0.85, 0.55), light: rgb(0.0, 0.40, 0.24)) }
  static var onAccent: Color { adaptive(rgb(0.06, 0.10, 0.13), light: .white) }
  static var border: Color { primary.opacity(0.14) }
  static let cardRadius: CGFloat = 14
  static let sidebarWidth: CGFloat = 198
}
