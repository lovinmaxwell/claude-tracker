import SwiftUI

/// The pixel creature from the Claude Tracker icon, drawn from the same 16x16 grid as
/// the original (`app/Services/PixelMascot.php`) so it stays identical across the apps.
struct MascotView: View {
    /// B is body, E is an eye, a dot is empty.
    private static let art = [
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

    private static let eye = Color(red: 0.106, green: 0.106, blue: 0.078)

    /// Side of one grid cell, in points.
    let cell: CGFloat

    var body: some View {
        Canvas { context, size in
            for (y, row) in Self.art.enumerated() {
                for (x, character) in row.enumerated() where character != "." {
                    // The art is centred on cell (8, 8), so it sits in the middle of the canvas.
                    let rect = CGRect(
                        x: size.width / 2 + CGFloat(x - 8) * cell,
                        y: size.height / 2 + CGFloat(y - 8) * cell,
                        width: cell + 0.5,
                        height: cell + 0.5
                    )
                    context.fill(Path(rect), with: .color(character == "E" ? Self.eye : WidgetPalette.session))
                }
            }
        }
        .frame(width: cell * 16, height: cell * 16)
    }
}
