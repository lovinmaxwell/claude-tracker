import AppKit
import QuartzCore
import SwiftUI

/// A borderless panel that never takes focus, so showing or clicking the
/// widget doesn't pull the user out of whatever they're typing in.
final class FloatingPanel: NSPanel {
    override var canBecomeKey: Bool { false }
    override var canBecomeMain: Bool { false }
}

/// The window's content view. It owns every mouse event so the widget behaves
/// the same wherever you hit it: hover expands, drag moves, double-click opens.
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

    func setCornerRadius(_ radius: CGFloat, animated: Bool) {
        wantsLayer = true
        guard let layer else { return }
        layer.masksToBounds = true
        if animated {
            let animation = CABasicAnimation(keyPath: "cornerRadius")
            animation.fromValue = layer.presentation()?.cornerRadius ?? layer.cornerRadius
            animation.toValue = radius
            animation.duration = WidgetLayout.animationDuration
            layer.add(animation, forKey: "cornerRadius")
        }
        layer.cornerRadius = radius
    }
}

/// Owns the floating window: placement, hover expand/collapse, drag memory.
final class WidgetPanelController: NSObject {
    var onDoubleClick: (() -> Void)?

    private let panel: FloatingPanel
    private let content = PanelContentView()
    private let hover = HoverState()
    private var pendingCollapse: DispatchWorkItem?
    private var isAnimating = false

    /// Top-left corner of the resting pill, in screen coordinates (y grows upward).
    private var anchor: NSPoint
    /// How far left/up the expanded card sits from the anchor to stay on screen.
    private var shift = NSPoint.zero

    private static let anchorXKey = "widgetAnchorX"
    private static let anchorYKey = "widgetAnchorY"

    var isVisible: Bool { panel.isVisible }

    init(store: UsageStore) {
        panel = FloatingPanel(
            contentRect: NSRect(origin: .zero, size: WidgetLayout.pill),
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

        let host = NSHostingView(rootView: WidgetRootView(store: store, hover: hover))
        host.frame = content.bounds
        host.autoresizingMask = [.width, .height]
        content.addSubview(host)

        content.setCornerRadius(WidgetLayout.pillRadius, animated: false)
        content.onHoverChange = { [weak self] inside in self?.handleHover(inside) }
        content.onDoubleClick = { [weak self] in self?.onDoubleClick?() }

        panel.setFrame(frame(for: WidgetLayout.pill), display: false)

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

    /// Right-click menu, available wherever on the widget the click lands.
    func setContextMenu(_ menu: NSMenu) {
        content.menu = menu
    }

    func hide() {
        pendingCollapse?.cancel()
        setExpanded(false, animated: false)
        panel.orderOut(nil)
    }

    func resetPosition() {
        anchor = WidgetPanelController.defaultAnchor()
        saveAnchor()
        setExpanded(false, animated: false)
        panel.setFrame(frame(for: WidgetLayout.pill), display: true)
    }

    // MARK: - Hover

    private func handleHover(_ inside: Bool) {
        pendingCollapse?.cancel()
        if inside {
            setExpanded(true, animated: true)
            return
        }
        let work = DispatchWorkItem { [weak self] in
            guard let self else { return }
            // Drag-and-drop can shuffle enter/exit events; trust where the pointer actually is.
            if !self.panel.frame.contains(NSEvent.mouseLocation) {
                self.setExpanded(false, animated: true)
            }
        }
        pendingCollapse = work
        DispatchQueue.main.asyncAfter(deadline: .now() + WidgetLayout.collapseDelay, execute: work)
    }

    private func setExpanded(_ expanded: Bool, animated: Bool) {
        guard hover.expanded != expanded else { return }
        hover.expanded = expanded
        let size = expanded ? WidgetLayout.card : WidgetLayout.pill
        shift = expanded ? shiftToFit(size) : .zero
        let target = frame(for: size)
        let alpha = expanded ? 1 : WidgetLayout.restingAlpha
        content.setCornerRadius(expanded ? WidgetLayout.cardRadius : WidgetLayout.pillRadius, animated: animated)

        guard animated, panel.isVisible else {
            panel.setFrame(target, display: true)
            panel.alphaValue = alpha
            return
        }
        isAnimating = true
        let panel = self.panel
        NSAnimationContext.runAnimationGroup({ context in
            context.duration = WidgetLayout.animationDuration
            panel.animator().setFrame(target, display: true)
            panel.animator().alphaValue = alpha
        }, completionHandler: { [weak self] in
            self?.isAnimating = false
            self?.panel.invalidateShadow()
        })
    }

    // MARK: - Geometry

    private func frame(for size: CGSize) -> NSRect {
        NSRect(
            x: anchor.x - shift.x,
            y: anchor.y + shift.y - size.height,
            width: size.width,
            height: size.height
        )
    }

    /// Slides the card left/up just enough to keep it inside the screen's usable area.
    private func shiftToFit(_ size: CGSize) -> NSPoint {
        let probe = NSPoint(x: anchor.x + 1, y: anchor.y - 1)
        guard let screen = NSScreen.screens.first(where: { $0.frame.contains(probe) }) ?? NSScreen.main else {
            return .zero
        }
        let area = screen.visibleFrame
        let margin = WidgetLayout.screenMargin

        let overflowX = max(0, anchor.x + size.width - (area.maxX - margin))
        let roomX = max(0, anchor.x - (area.minX + margin))

        let bottom = anchor.y - size.height
        let overflowY = max(0, (area.minY + margin) - bottom)
        let roomY = max(0, (area.maxY - margin) - anchor.y)

        return NSPoint(x: min(overflowX, roomX), y: min(overflowY, roomY))
    }

    @objc private func windowDidMove() {
        guard !isAnimating else { return }
        anchor = NSPoint(x: panel.frame.minX + shift.x, y: panel.frame.maxY - shift.y)
        saveAnchor()
    }

    /// A monitor was unplugged: don't leave the widget stranded off-screen.
    @objc private func screensChanged() {
        let probe = NSPoint(x: anchor.x + 1, y: anchor.y - 1)
        if !NSScreen.screens.contains(where: { $0.frame.contains(probe) }) {
            resetPosition()
        }
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
        let point = NSPoint(x: defaults.double(forKey: anchorXKey), y: defaults.double(forKey: anchorYKey))
        let probe = NSPoint(x: point.x + 1, y: point.y - 1)
        return NSScreen.screens.contains(where: { $0.frame.contains(probe) }) ? point : nil
    }

    /// First run: top-right corner of the main screen, clear of the menu bar.
    private static func defaultAnchor() -> NSPoint {
        let area = NSScreen.main?.visibleFrame ?? NSRect(x: 0, y: 0, width: 1440, height: 900)
        return NSPoint(
            x: area.maxX - WidgetLayout.pill.width - WidgetLayout.defaultInset,
            y: area.maxY - WidgetLayout.defaultInset
        )
    }
}
