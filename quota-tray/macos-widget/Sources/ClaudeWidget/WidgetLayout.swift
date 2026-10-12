import Foundation

/// Sizes and timings shared by the AppKit window and the SwiftUI content inside it.
enum WidgetLayout {
    /// The widget is always this card; only its opacity changes.
    static let card = CGSize(width: 336, height: 204)
    static let cardRadius: CGFloat = 22

    /// Window opacity until the pointer is over the widget.
    static let restingAlpha: CGFloat = 0.2

    static let defaultInset: CGFloat = 16
    static let fadeDuration: TimeInterval = 0.2
    /// Grace period before fading back, so a pointer grazing the edge doesn't flicker.
    static let restDelay: TimeInterval = 0.25
}
