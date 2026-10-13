import ClaudeUsageCore
import SwiftUI

/// The pixel creature from the Claude Tracker icon, drawn from the same 16x16 grid as
/// the original `PixelMascot` in Bhargav Detroja's Claude Tracker (see the root README credits), so it
/// stays identical across the apps.
/// Like the original, it fills with terracotta from the feet up as usage grows.
enum MascotArt {
    /// B is body, E is an eye, a dot is empty.
    static let art = [
        "................",
        "................",
        "...BBBBBBBBBB...",
        "...BBBBBBBBBB...",
        "...BEEBBBBEEB...",
        "...BEEBBBBEEB...",
        ".BBBBBBBBBBBBBB.",
        ".BBBBBBBBBBBBBB.",
        ".BBBBBBBBBBBBBB.",
        "...BBBBBBBBBB...",
        "...BBBBBBBBBB...",
        "...BB.BB.BB.BB..",
        "...BB.BB.BB.BB..",
        "...BB.BB.BB.BB..",
        "................",
        "................",
    ]

    /// Terracotta: the spent part of the body.
    static let spent = Color(red: 0.788, green: 0.392, blue: 0.259)
    /// Pale body colour: the unspent part.
    static let unspent = Color(red: 0.875, green: 0.824, blue: 0.769)
    static let eye = Color(red: 0.173, green: 0.173, blue: 0.173)

    /// The creature occupies grid rows 2 through 13, which is what the fill spans.
    private static let lastBodyRow = 13

    /// Draws the creature centred on `center`. `fill` is percent spent (0...100), filled from the
    /// feet up; nil means unknown and shows the unspent body, never a made-up reading.
    static func draw(in context: inout GraphicsContext, center: CGPoint, cell: CGFloat, fill: Double?) {
        let spentRows = IconFill.litSteps(fill)
        for (y, row) in art.enumerated() {
            for (x, character) in row.enumerated() where character != "." {
                let color: Color
                if character == "E" {
                    color = eye
                } else {
                    color = (lastBodyRow - y) < spentRows ? spent : unspent
                }
                // The art is centred on cell (8, 8); the small overlap hides hairline seams between cells.
                let rect = CGRect(
                    x: center.x + CGFloat(x - 8) * cell,
                    y: center.y + CGFloat(y - 8) * cell,
                    width: cell + 0.5,
                    height: cell + 0.5
                )
                context.fill(Path(rect), with: .color(color))
            }
        }
    }
}

/// The creature on its own, used in the middle of the card's rings.
struct MascotView: View {
    /// Side of one grid cell, in points.
    let cell: CGFloat
    /// Percent spent, 0...100; nil when unknown.
    let fill: Double?

    var body: some View {
        Canvas { context, size in
            MascotArt.draw(
                in: &context,
                center: CGPoint(x: size.width / 2, y: size.height / 2),
                cell: cell,
                fill: fill
            )
        }
        .frame(width: cell * 16, height: cell * 16)
    }
}

/// The Claude Tracker icon: a ring of 12 rounded pills around the creature. The pills fill
/// clockwise from the top as usage grows (the last quarter turns coral as a warning) while the
/// creature fills from its feet up, exactly as in the original menu bar icon.
struct TrackerIconView: View {
    /// Percent spent, 0...100; nil when unknown.
    let used: Double?
    let isStale: Bool
    let size: CGFloat

    private static let pillCount = 12
    private static let coral = Color(red: 0.851, green: 0.294, blue: 0.204)
    private static let emptyPill = Color(red: 0.875, green: 0.824, blue: 0.769).opacity(0.45)

    var body: some View {
        Canvas { context, canvasSize in
            let center = CGPoint(x: canvasSize.width / 2, y: canvasSize.height / 2)
            MascotArt.draw(in: &context, center: center, cell: size * 0.75 / 16, fill: used)
            drawPills(in: &context, center: center)
        }
        .frame(width: size, height: size)
    }

    /// Twelve short arcs around the edge; the proportions match the original 32-point icon, scaled.
    private func drawPills(in context: inout GraphicsContext, center: CGPoint) {
        let scale = size / 32
        let radius = 13.4 * scale
        let thickness = 2 * 1.35 * scale
        let filled = IconFill.litSteps(used)
        let step = 2 * Double.pi / Double(Self.pillCount)

        for index in 0..<Self.pillCount {
            let middle = -Double.pi / 2 + (Double(index) + 0.5) * step
            var path = Path()
            let segments = 6
            for segment in 0...segments {
                let angle = middle - 0.10 + 0.20 * Double(segment) / Double(segments)
                let point = CGPoint(
                    x: center.x + radius * CGFloat(cos(angle)),
                    y: center.y + radius * CGFloat(sin(angle))
                )
                if segment == 0 {
                    path.move(to: point)
                } else {
                    path.addLine(to: point)
                }
            }

            var color = Self.emptyPill
            if index < filled {
                let base = index >= 9 ? Self.coral : MascotArt.spent
                color = base.opacity(isStale ? 0.55 : 1)
            }
            context.stroke(
                path,
                with: .color(color),
                style: StrokeStyle(lineWidth: thickness, lineCap: .round, lineJoin: .round)
            )
        }
    }
}
