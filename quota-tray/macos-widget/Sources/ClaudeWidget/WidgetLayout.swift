import Foundation

/// Sizes and timings shared by the AppKit window and the SwiftUI content inside it.
enum WidgetLayout {
    /// Resting state: just the rings. Hovered state: the full card.
    static let rest = CGSize(width: 100, height: 100)
    static let card = CGSize(width: 336, height: 204)
    static let restRadius: CGFloat = 50
    static let cardRadius: CGFloat = 22

    /// Window opacity until the pointer is over the widget. Bare rings with no
    /// backing vanish into a wallpaper at 20%, so the resting rings sit higher.
    static let restingAlpha: CGFloat = 0.6

    static let screenMargin: CGFloat = 12
    static let defaultInset: CGFloat = 16
    static let animationDuration: TimeInterval = 0.2
    /// Grace period before collapsing, so a pointer grazing the edge doesn't flicker.
    static let collapseDelay: TimeInterval = 0.3
}
