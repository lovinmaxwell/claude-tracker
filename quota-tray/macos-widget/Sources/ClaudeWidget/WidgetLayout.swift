import Foundation

/// Sizes and timings shared by the AppKit window and the SwiftUI content inside it.
enum WidgetLayout {
    /// Resting pill and hovered card; the window is resized between the two.
    static let pill = CGSize(width: 212, height: 64)
    static let card = CGSize(width: 336, height: 204)
    static let pillRadius: CGFloat = 32
    static let cardRadius: CGFloat = 22

    /// Window opacity until the pointer is over the widget.
    static let restingAlpha: CGFloat = 0.2

    static let screenMargin: CGFloat = 12
    static let defaultInset: CGFloat = 16
    static let animationDuration: TimeInterval = 0.2
    /// Grace period before collapsing, so a pointer grazing the edge doesn't flicker.
    static let collapseDelay: TimeInterval = 0.3
}
