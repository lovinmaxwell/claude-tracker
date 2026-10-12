import AppKit
import SwiftUI

/// A borderless panel that never takes focus, so showing or clicking the
/// widget doesn't pull the user out of whatever they're typing in.
final class FloatingPanel: NSPanel {
    override var canBecomeKey: Bool { false }
    override var canBecomeMain: Bool { false }
}

/// The window's content view. It owns every mouse event so the widget behaves
/// the same wherever you hit it: hover brightens, drag moves, double-click opens.
final class PanelContentView: NSView {
    var onHoverChange: ((Bool) -> Void)?
    var onDoubleClick: (() -> Void)?

    override var mouseDownCanMoveWindow: Bool { false }

    override func hitTest(_ point: NSPoint) -> NSView? {
        bounds.contains(convert(point, from: superview)) ? self : nil
    }

    override func acceptsFirstMouse(for event: NSEvent?) -> Bool { true }

    override func mouseDown(with event: NSEvent) {
        if event.clickCount >= 2 {
            onDoubleClick?()
        } else {
            // Native drag: tracks the pointer until the button is released.
            window?.performDrag(with: event)
        }
    }

    override func updateTrackingAreas() {
        super.updateTrackingAreas()
        for area in trackingAreas {
            removeTrackingArea(area)
        }
        // activeAlways: the app is never frontmost, yet hover must still register.
        addTrackingArea(
            NSTrackingArea(
                rect: bounds,
                options: [.mouseEnteredAndExited, .activeAlways, .inVisibleRect],
                owner: self,
                userInfo: nil
            )
        )
    }

    override func mouseEntered(with event: NSEvent) { onHoverChange?(true) }
    override func mouseExited(with event: NSEvent) { onHoverChange?(false) }

    func setCornerRadius(_ radius: CGFloat) {
        wantsLayer = true
        guard let layer else { return }
        layer.masksToBounds = true
        layer.cornerRadius = radius
    }
}

/// Owns the floating window: placement, the 20% to 100% fade on hover, drag memory.
final class WidgetPanelController: NSObject {
    var onDoubleClick: (() -> Void)?

    private let panel: FloatingPanel
    private let content = PanelContentView()
    private var pendingRest: DispatchWorkItem?
    private var isHovered = false

    /// Top-left corner of the window in screen coordinates (y grows upward).
    private var anchor: NSPoint

    private static let anchorXKey = "widgetAnchorX"
    private static let anchorYKey = "widgetAnchorY"

    var isVisible: Bool { panel.isVisible }

    init(store: UsageStore) {
        panel = FloatingPanel(
            contentRect: NSRect(origin: .zero, size: WidgetLayout.card),
            styleMask: [.borderless, .nonactivatingPanel],
            backing: .buffered,
            defer: false
        )
        anchor = WidgetPanelController.loadAnchor() ?? WidgetPanelController.defaultAnchor()
        super.init()

        panel.level = .floating
        panel.isFloatingPanel = true
        panel.isOpaque = false
        panel.backgroundColor = .clear
        panel.hasShadow = true
        panel.hidesOnDeactivate = false
        panel.isReleasedWhenClosed = false
        panel.collectionBehavior = [.canJoinAllSpaces, .fullScreenAuxiliary, .stationary]
        panel.alphaValue = WidgetLayout.restingAlpha
        panel.contentView = content

        // Frosted glass underneath, SwiftUI on top. A dedicated NSVisualEffectView
        // (state .active) keeps the blur even though this app is never frontmost.
        let glass = NSVisualEffectView(frame: content.bounds)
        glass.material = .popover
        glass.blendingMode = .behindWindow
        glass.state = .active
        glass.autoresizingMask = [.width, .height]
        content.addSubview(glass)

        let host = NSHostingView(rootView: WidgetRootView(store: store))
        host.frame = content.bounds
        host.autoresizingMask = [.width, .height]
        content.addSubview(host)

        content.setCornerRadius(WidgetLayout.cardRadius)
        content.onHoverChange = { [weak self] inside in self?.handleHover(inside) }
        content.onDoubleClick = { [weak self] in self?.onDoubleClick?() }

        panel.setFrame(frame(), display: false)

        NotificationCenter.default.addObserver(
            self, selector: #selector(windowDidMove),
            name: NSWindow.didMoveNotification, object: panel)
        NotificationCenter.default.addObserver(
            self, selector: #selector(screensChanged),
            name: NSApplication.didChangeScreenParametersNotification, object: nil)
    }

    func show() {
        panel.orderFrontRegardless()
    }

    func hide() {
        pendingRest?.cancel()
        isHovered = false
        panel.alphaValue = WidgetLayout.restingAlpha
        panel.orderOut(nil)
    }

    /// Right-click menu, available wherever on the widget the click lands.
    func setContextMenu(_ menu: NSMenu) {
        content.menu = menu
    }

    func resetPosition() {
        anchor = WidgetPanelController.defaultAnchor()
        saveAnchor()
        panel.setFrame(frame(), display: true)
    }

    // MARK: - Hover

    private func handleHover(_ inside: Bool) {
        pendingRest?.cancel()
        if inside {
            setHovered(true)
            return
        }
        let work = DispatchWorkItem { [weak self] in
            guard let self else { return }
            // Trust where the pointer actually is, not a stray exit event.
            if !self.panel.frame.contains(NSEvent.mouseLocation) {
                self.setHovered(false)
            }
        }
        pendingRest = work
        DispatchQueue.main.asyncAfter(deadline: .now() + WidgetLayout.restDelay, execute: work)
    }

    private func setHovered(_ hovered: Bool) {
        guard isHovered != hovered else { return }
        isHovered = hovered
        let alpha = hovered ? 1 : WidgetLayout.restingAlpha
        let panel = self.panel
        NSAnimationContext.runAnimationGroup { context in
            context.duration = WidgetLayout.fadeDuration
            panel.animator().alphaValue = alpha
        }
    }

    // MARK: - Geometry

    private func frame() -> NSRect {
        NSRect(
            x: anchor.x,
            y: anchor.y - WidgetLayout.card.height,
            width: WidgetLayout.card.width,
            height: WidgetLayout.card.height
        )
    }

    @objc private func windowDidMove() {
        anchor = NSPoint(x: panel.frame.minX, y: panel.frame.maxY)
        saveAnchor()
    }

    /// A monitor was unplugged: don't leave the widget stranded off-screen.
    @objc private func screensChanged() {
        if WidgetPanelController.fitted(anchor) == nil {
            resetPosition()
        }
    }

    /// Pulls a top-left anchor back inside the usable area of the screen it sits on;
    /// nil when it is on no connected screen at all.
    private static func fitted(_ anchor: NSPoint) -> NSPoint? {
        let probe = NSPoint(x: anchor.x + 1, y: anchor.y - 1)
        guard let screen = NSScreen.screens.first(where: { $0.frame.contains(probe) }) else {
            return nil
        }
        let area = screen.visibleFrame
        let size = WidgetLayout.card
        let x = min(max(anchor.x, area.minX), max(area.minX, area.maxX - size.width))
        let y = min(max(anchor.y, area.minY + size.height), area.maxY)
        return NSPoint(x: x, y: y)
    }

    // MARK: - Persistence

    private func saveAnchor() {
        let defaults = UserDefaults.standard
        defaults.set(anchor.x, forKey: WidgetPanelController.anchorXKey)
        defaults.set(anchor.y, forKey: WidgetPanelController.anchorYKey)
    }

    private static func loadAnchor() -> NSPoint? {
        let defaults = UserDefaults.standard
        guard
            defaults.object(forKey: anchorXKey) != nil,
            defaults.object(forKey: anchorYKey) != nil
        else { return nil }
        return fitted(NSPoint(x: defaults.double(forKey: anchorXKey), y: defaults.double(forKey: anchorYKey)))
    }

    /// First run: top-right corner of the main screen, clear of the menu bar.
    private static func defaultAnchor() -> NSPoint {
        let area = NSScreen.main?.visibleFrame ?? NSRect(x: 0, y: 0, width: 1440, height: 900)
        return NSPoint(
            x: area.maxX - WidgetLayout.card.width - WidgetLayout.defaultInset,
            y: area.maxY - WidgetLayout.defaultInset
        )
    }
}
