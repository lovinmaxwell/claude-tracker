import AppKit
import ClaudeUsageCore

/// The Claude Tracker icon as an `NSImage` for the menu bar. It is drawn with plain AppKit rather than
/// by rendering the SwiftUI view, because SwiftUI's `ImageRenderer` is main-actor only. The geometry is
/// the same as `TrackerIconView`: 12 pills around the creature, which fills from its feet up.
enum TrackerIconImage {
    private static let spent = NSColor(srgbRed: 0.788, green: 0.392, blue: 0.259, alpha: 1)
    private static let unspent = NSColor(srgbRed: 0.875, green: 0.824, blue: 0.769, alpha: 1)
    private static let eye = NSColor(srgbRed: 0.173, green: 0.173, blue: 0.173, alpha: 1)
    private static let coral = NSColor(srgbRed: 0.851, green: 0.294, blue: 0.204, alpha: 1)

    /// A square image `size` points wide. Resolution-independent: macOS redraws it at the screen's scale.
    static func make(used: Double?, isStale: Bool, size: CGFloat) -> NSImage {
        // flipped: true gives a top-left origin, matching the y-down geometry used everywhere else.
        NSImage(size: NSSize(width: size, height: size), flipped: true) { _ in
            drawCreature(used: used, size: size)
            drawPills(used: used, isStale: isStale, size: size)
            return true
        }
    }

    /// The creature, three quarters of the icon wide, filled from the feet up.
    private static func drawCreature(used: Double?, size: CGFloat) {
        let center = NSPoint(x: size / 2, y: size / 2)
        let cell = size * 0.75 / 16
        let spentRows = IconFill.litSteps(used)

        for (y, row) in MascotArt.art.enumerated() {
            for (x, character) in row.enumerated() where character != "." {
                if character == "E" {
                    eye.setFill()
                } else if 13 - y < spentRows {
                    spent.setFill()
                } else {
                    unspent.setFill()
                }
                // The small overlap hides hairline seams between cells.
                NSBezierPath.fill(NSRect(
                    x: center.x + CGFloat(x - 8) * cell,
                    y: center.y + CGFloat(y - 8) * cell,
                    width: cell + 0.3,
                    height: cell + 0.3
                ))
            }
        }
    }

    /// Twelve short arcs around the edge; the proportions match the original 32-point icon, scaled.
    private static func drawPills(used: Double?, isStale: Bool, size: CGFloat) {
        let center = NSPoint(x: size / 2, y: size / 2)
        let scale = size / 32
        let radius = 13.4 * scale
        let filled = IconFill.litSteps(used)
        let step = 2 * Double.pi / Double(IconFill.steps)

        for index in 0..<IconFill.steps {
            let middle = -Double.pi / 2 + (Double(index) + 0.5) * step
            let path = NSBezierPath()
            path.lineWidth = 2 * 1.35 * scale
            path.lineCapStyle = .round
            let segments = 6
            for segment in 0...segments {
                let angle = middle - 0.10 + 0.20 * Double(segment) / Double(segments)
                let point = NSPoint(
                    x: center.x + radius * CGFloat(cos(angle)),
                    y: center.y + radius * CGFloat(sin(angle))
                )
                if segment == 0 {
                    path.move(to: point)
                } else {
                    path.line(to: point)
                }
            }

            if index < filled {
                (index >= 9 ? coral : spent).withAlphaComponent(isStale ? 0.55 : 1).setStroke()
            } else {
                unspent.withAlphaComponent(0.45).setStroke()
            }
            path.stroke()
        }
    }
}
