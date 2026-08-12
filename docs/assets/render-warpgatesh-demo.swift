#!/usr/bin/env swift

import AppKit
import Foundation
import ImageIO
import UniformTypeIdentifiers

let canvasWidth = 960
let canvasHeight = 540
let frameCount = 60
let frameDelay = 0.2
let outputPath = CommandLine.arguments.dropFirst().first ?? "docs/assets/warpgatesh-demo.gif"

struct TerminalLine {
    let text: String
    let color: NSColor
    let top: CGFloat
    let size: CGFloat
    let appearsAt: Int
}

func color(_ hex: UInt32, alpha: CGFloat = 1) -> NSColor {
    NSColor(
        calibratedRed: CGFloat((hex >> 16) & 0xff) / 255,
        green: CGFloat((hex >> 8) & 0xff) / 255,
        blue: CGFloat(hex & 0xff) / 255,
        alpha: alpha
    )
}

func topRect(x: CGFloat, y: CGFloat, width: CGFloat, height: CGFloat) -> NSRect {
    NSRect(
        x: x,
        y: CGFloat(canvasHeight) - y - height,
        width: width,
        height: height
    )
}

func fill(_ rect: NSRect, with fillColor: NSColor, radius: CGFloat = 0) {
    fillColor.setFill()
    NSBezierPath(roundedRect: rect, xRadius: radius, yRadius: radius).fill()
}

func drawText(
    _ text: String,
    x: CGFloat,
    top: CGFloat,
    size: CGFloat,
    textColor: NSColor,
    weight: NSFont.Weight = .regular
) {
    let font = NSFont.monospacedSystemFont(ofSize: size, weight: weight)
    let attributes: [NSAttributedString.Key: Any] = [
        .font: font,
        .foregroundColor: textColor
    ]
    let measured = (text as NSString).size(withAttributes: attributes)
    (text as NSString).draw(
        at: NSPoint(x: x, y: CGFloat(canvasHeight) - top - measured.height),
        withAttributes: attributes
    )
}

func drawCenteredText(
    _ text: String,
    top: CGFloat,
    size: CGFloat,
    textColor: NSColor
) {
    let font = NSFont.monospacedSystemFont(ofSize: size, weight: .medium)
    let attributes: [NSAttributedString.Key: Any] = [
        .font: font,
        .foregroundColor: textColor
    ]
    let measured = (text as NSString).size(withAttributes: attributes)
    (text as NSString).draw(
        at: NSPoint(
            x: (CGFloat(canvasWidth) - measured.width) / 2,
            y: CGFloat(canvasHeight) - top - measured.height
        ),
        withAttributes: attributes
    )
}

let prompt = color(0x79c0ff)
let foreground = color(0xc9d1d9)
let success = color(0x7ee787)
let remotePrompt = color(0xd2a8ff)

let lines = [
    TerminalLine(text: "$ warpgatesh ls", color: prompt, top: 112, size: 23, appearsAt: 5),
    TerminalLine(
        text: "app-01       lab    Application server",
        color: foreground,
        top: 151,
        size: 21,
        appearsAt: 10
    ),
    TerminalLine(
        text: "db-primary   lab    PostgreSQL primary",
        color: foreground,
        top: 184,
        size: 21,
        appearsAt: 14
    ),
    TerminalLine(text: "$ warpgatesh sync", color: prompt, top: 233, size: 23, appearsAt: 20),
    TerminalLine(
        text: "Synchronized 2 SSH target(s) from 1 profile(s): +0, -0",
        color: success,
        top: 272,
        size: 19,
        appearsAt: 25
    ),
    TerminalLine(
        text: "$ warpgatesh app-01",
        color: prompt,
        top: 321,
        size: 23,
        appearsAt: 35
    ),
    TerminalLine(
        text: "Last login: Tue Aug 12 10:42:18",
        color: foreground,
        top: 360,
        size: 20,
        appearsAt: 42
    ),
    TerminalLine(text: "app-01:~ $", color: remotePrompt, top: 401, size: 23, appearsAt: 47)
]

let outputURL = URL(fileURLWithPath: outputPath)
guard let destination = CGImageDestinationCreateWithURL(
    outputURL as CFURL,
    UTType.gif.identifier as CFString,
    frameCount,
    nil
) else {
    fatalError("Could not create GIF destination at \(outputPath)")
}

let gifProperties: CFDictionary = [
    kCGImagePropertyGIFDictionary: [
        kCGImagePropertyGIFLoopCount: 0
    ]
] as CFDictionary
CGImageDestinationSetProperties(destination, gifProperties)

for frameIndex in 0..<frameCount {
    let image = NSImage(size: NSSize(width: canvasWidth, height: canvasHeight))
    image.lockFocus()

    fill(NSRect(x: 0, y: 0, width: canvasWidth, height: canvasHeight), with: color(0x0d1117))
    fill(topRect(x: 52, y: 42, width: 856, height: 446), with: color(0x000000, alpha: 0.35), radius: 15)
    fill(topRect(x: 60, y: 34, width: 840, height: 446), with: color(0x161b22), radius: 13)
    fill(topRect(x: 60, y: 34, width: 840, height: 48), with: color(0x21262d), radius: 13)
    fill(topRect(x: 60, y: 70, width: 840, height: 12), with: color(0x21262d))
    fill(topRect(x: 60, y: 82, width: 840, height: 2), with: color(0x30363d))

    fill(topRect(x: 83, y: 51, width: 14, height: 14), with: color(0xff5f57), radius: 7)
    fill(topRect(x: 112, y: 51, width: 14, height: 14), with: color(0xfebc2e), radius: 7)
    fill(topRect(x: 141, y: 51, width: 14, height: 14), with: color(0x28c840), radius: 7)
    drawCenteredText(
        "WarpgateSH — SSH targets, one command away",
        top: 48,
        size: 17,
        textColor: color(0x8b949e)
    )

    for line in lines where frameIndex >= line.appearsAt {
        drawText(
            line.text,
            x: 90,
            top: line.top,
            size: line.size,
            textColor: line.color
        )
    }

    if frameIndex >= 47 && ((frameIndex - 47) / 3).isMultiple(of: 2) {
        fill(topRect(x: 245, y: 404, width: 13, height: 24), with: foreground)
    }

    drawText("DEMO DATA", x: 793, top: 451, size: 13, textColor: color(0x6e7681), weight: .medium)
    image.unlockFocus()

    var proposedRect = NSRect(x: 0, y: 0, width: canvasWidth, height: canvasHeight)
    guard let cgImage = image.cgImage(forProposedRect: &proposedRect, context: nil, hints: nil) else {
        fatalError("Could not render frame \(frameIndex)")
    }

    let frameProperties: CFDictionary = [
        kCGImagePropertyGIFDictionary: [
            kCGImagePropertyGIFDelayTime: frameDelay
        ]
    ] as CFDictionary
    CGImageDestinationAddImage(destination, cgImage, frameProperties)
}

guard CGImageDestinationFinalize(destination) else {
    fatalError("Could not finalize GIF at \(outputPath)")
}

print("Rendered \(outputPath)")
