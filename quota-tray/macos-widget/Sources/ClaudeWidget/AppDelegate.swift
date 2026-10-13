import AppKit
import Combine
import SwiftUI
import ClaudeUsageCore

final class AppDelegate: NSObject, NSApplicationDelegate, NSMenuDelegate {
    private enum Tag: Int {
        case toggleWidget = 1
        case doubleClickTerminal = 10
        case doubleClickApp = 11
        case launchAtLogin = 20
        case autoSession = 40
        case lastAutoSession = 41
        case crashRestart = 50
    }

    private let store = UsageStore()
    private var controller: WidgetPanelController?
    private var statusItem: NSStatusItem?
    private var storeObserver: AnyCancellable?
    private let popover = NSPopover()
    private let autoSession = AutoSessionRunner()
    private static let hiddenKey = "widgetHidden"

    func applicationDidFinishLaunching(_ notification: Notification) {
        if isDuplicateInstance() {
            NSApp.terminate(nil)
            return
        }

        let controller = WidgetPanelController(store: store)
        controller.onDoubleClick = { [weak self] in self?.performDoubleClick() }
        self.controller = controller
        if !UserDefaults.standard.bool(forKey: AppDelegate.hiddenKey) {
            controller.show()
        }

        // Right-clicking the widget and the menu bar item share one menu.
        controller.setContextMenu(makeMenu())
        installStatusItem()

        // After each reading: start the next session if the 5-hour window has ended.
        store.onUpdate = { [weak self] in self?.evaluateAutoSession() }

        // Keep the menu bar tooltip in step with each new reading.
        storeObserver = store.objectWillChange
            .receive(on: DispatchQueue.main)
            .sink { [weak self] _ in self?.updateTooltip() }
        store.start()
        NSWorkspace.shared.notificationCenter.addObserver(
            self, selector: #selector(systemDidWake),
            name: NSWorkspace.didWakeNotification, object: nil)

        LaunchAtLogin.enableOnFirstRun()
    }

    /// The login item and a manual launch must not leave two widgets on screen.
    private func isDuplicateInstance() -> Bool {
        guard let identifier = Bundle.main.bundleIdentifier else { return false }
        return NSRunningApplication.runningApplications(withBundleIdentifier: identifier).count > 1
    }

    @objc private func systemDidWake() {
        store.refresh()
    }

    // MARK: - Menu

    private func installStatusItem() {
        let item = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
        // Starts as the empty tracker icon; updateTooltip() redraws it with each reading.
        item.button?.image = trackerIcon(used: nil, isStale: false)
        // Left-click shows the same card as the floating widget; right-click shows the menu.
        item.button?.target = self
        item.button?.action = #selector(statusItemClicked(_:))
        item.button?.sendAction(on: [.leftMouseUp, .rightMouseUp])
        statusItem = item

        popover.behavior = .transient
        popover.contentSize = NSSize(width: WidgetLayout.card.width, height: WidgetLayout.card.height)
        popover.contentViewController = NSHostingController(rootView: PopoverRootView(store: store))
    }

    @objc private func statusItemClicked(_ sender: NSStatusBarButton) {
        let event = NSApp.currentEvent
        let wantsMenu = event?.type == .rightMouseUp || (event?.modifierFlags.contains(.control) ?? false)
        if wantsMenu {
            popover.performClose(nil)
            showMenu()
        } else if popover.isShown {
            popover.performClose(nil)
        } else {
            store.refresh()
            popover.show(relativeTo: sender.bounds, of: sender, preferredEdge: .minY)
            NSApp.activate(ignoringOtherApps: true)
        }
    }

    /// Assigning the menu just for the click is the standard way to get both a
    /// left-click action and a right-click menu on a status item.
    private func showMenu() {
        guard let item = statusItem else { return }
        item.menu = makeMenu()
        item.button?.performClick(nil)
        item.menu = nil
    }

    private func makeMenu() -> NSMenu {
        let menu = NSMenu()
        menu.delegate = self
        menu.addItem(item("Open Claude Code", #selector(openClaudeCode)))
        menu.addItem(item("Open Claude App", #selector(openClaudeApp)))
        menu.addItem(.separator())
        menu.addItem(item("Refresh Now", #selector(refreshNow)))
        menu.addItem(item("Hide Widget", #selector(toggleWidget), tag: .toggleWidget))
        menu.addItem(item("Reset Position", #selector(resetPosition)))
        menu.addItem(.separator())
        menu.addItem(item("Double-Click Opens Claude Code", #selector(chooseTerminal), tag: .doubleClickTerminal))
        menu.addItem(item("Double-Click Opens Claude App", #selector(chooseDesktopApp), tag: .doubleClickApp))
        menu.addItem(item("Launch at Login", #selector(toggleLaunchAtLogin), tag: .launchAtLogin))
        menu.addItem(item("Restart if It Crashes", #selector(toggleCrashRestart), tag: .crashRestart))
        menu.addItem(.separator())
        menu.addItem(item("Auto-Start New Session", #selector(toggleAutoSession), tag: .autoSession))
        menu.addItem(item("Set Trigger Message…", #selector(editTriggerMessage)))
        menu.addItem(item("Send Trigger Now", #selector(sendTriggerNow)))
        let last = NSMenuItem(title: "No auto-start yet", action: nil, keyEquivalent: "")
        last.tag = Tag.lastAutoSession.rawValue
        last.isEnabled = false
        menu.addItem(last)
        menu.addItem(.separator())
        menu.addItem(item("Quit Claude Widget", #selector(quit), key: "q"))
        return menu
    }

    private func item(_ title: String, _ action: Selector, tag: Tag? = nil, key: String = "") -> NSMenuItem {
        let menuItem = NSMenuItem(title: title, action: action, keyEquivalent: key)
        menuItem.target = self
        if let tag { menuItem.tag = tag.rawValue }
        return menuItem
    }

    // MARK: - Usage rows

    /// One line each: overall status, every usage window, and how fresh the reading is.
    private func usageRows() -> [String] {
        let now = Date()
        let snapshot = store.snapshot
        let status = UsageAnalysis.summarize(
            snapshot: snapshot,
            isStale: snapshot != nil && store.lastError != nil,
            error: store.lastError,
            now: now
        )
        var rows = [status.detail.map { "\(status.headline) · \($0)" } ?? status.headline]
        for window in snapshot?.windows ?? [] {
            var line = "\(window.label): \(Formatters.percent(window.used))"
            if let countdown = Formatters.countdown(to: window.resetsAt, now: now) {
                line += " · resets in \(countdown)"
                if let clock = Formatters.resetClock(window.resetsAt, now: now) {
                    line += " (\(clock))"
                }
            }
            rows.append(line)
        }
        if let age = Formatters.age(of: snapshot?.fetchedAt, now: now) {
            rows.append("Updated \(age)")
        }
        return rows
    }

    private func updateTooltip() {
        statusItem?.button?.toolTip = usageRows().joined(separator: "\n")
        let stale = store.snapshot != nil && store.lastError != nil
        statusItem?.button?.image = trackerIcon(
            used: UsageAnalysis.peak(store.snapshot?.windows ?? [])?.used,
            isStale: stale
        )
    }

    /// The Claude Tracker icon for the menu bar (18 points, full colour rather than a template).
    private func trackerIcon(used: Double?, isStale: Bool) -> NSImage? {
        TrackerIconImage.make(used: used, isStale: isStale, size: 18)
    }

    func menuNeedsUpdate(_ menu: NSMenu) {
        let terminal = ClaudeLauncher.preferredTarget == .terminal
        for menuItem in menu.items {
            switch Tag(rawValue: menuItem.tag) {
            case .toggleWidget:
                menuItem.title = (controller?.isVisible ?? false) ? "Hide Widget" : "Show Widget"
            case .doubleClickTerminal:
                menuItem.state = terminal ? .on : .off
            case .doubleClickApp:
                menuItem.state = terminal ? .off : .on
            case .launchAtLogin:
                menuItem.state = LaunchAtLogin.isEnabled ? .on : .off
            case .crashRestart:
                menuItem.state = CrashRestart.isEnabled ? .on : .off
            case .autoSession:
                menuItem.state = AutoSessionRunner.isEnabled ? .on : .off
            case .lastAutoSession:
                menuItem.title = lastAutoSessionTitle()
            case nil:
                break
            }
        }
    }

    // MARK: - Auto-start

    private func evaluateAutoSession() {
        autoSession.startIfNeeded(
            snapshot: store.snapshot,
            readingFailed: store.lastError != nil,
            now: Date()
        ) { [weak self] in self?.store.refresh() }
    }

    private func lastAutoSessionTitle() -> String {
        guard let at = AutoSessionRunner.lastAttempt else { return "No auto-start yet" }
        let time = Formatters.resetClock(at, now: Date()) ?? ""
        let result = AutoSessionRunner.lastResult ?? ""
        return "Last auto-start: \(time) · \(result)"
    }

    @objc private func toggleAutoSession() {
        AutoSessionRunner.isEnabled.toggle()
        if AutoSessionRunner.isEnabled {
            evaluateAutoSession()
        }
    }

    @objc private func sendTriggerNow() {
        autoSession.run { [weak self] in self?.store.refresh() }
    }

    @objc private func editTriggerMessage() {
        let alert = NSAlert()
        alert.messageText = "Trigger message"
        alert.informativeText = "Sent as claude -p \"…\" when your 5-hour session ends, to start the next one."
        let field = NSTextField(frame: NSRect(x: 0, y: 0, width: 320, height: 24))
        field.stringValue = AutoSessionRunner.message
        alert.accessoryView = field
        alert.addButton(withTitle: "Save")
        alert.addButton(withTitle: "Cancel")
        NSApp.activate(ignoringOtherApps: true)
        if alert.runModal() == .alertFirstButtonReturn {
            AutoSessionRunner.message = field.stringValue
        }
    }

    // MARK: - Actions

    private func performDoubleClick() {
        ClaudeLauncher.open(ClaudeLauncher.preferredTarget)
    }

    @objc private func openClaudeCode() { ClaudeLauncher.openClaudeCodeInTerminal() }
    @objc private func openClaudeApp() { ClaudeLauncher.openDesktopApp() }
    @objc private func refreshNow() { store.refresh() }
    @objc private func resetPosition() { controller?.resetPosition() }
    @objc private func chooseTerminal() { ClaudeLauncher.preferredTarget = .terminal }
    @objc private func chooseDesktopApp() { ClaudeLauncher.preferredTarget = .desktopApp }
    @objc private func quit() { NSApp.terminate(nil) }

    @objc private func toggleWidget() {
        guard let controller else { return }
        if controller.isVisible {
            controller.hide()
            UserDefaults.standard.set(true, forKey: AppDelegate.hiddenKey)
        } else {
            controller.show()
            UserDefaults.standard.set(false, forKey: AppDelegate.hiddenKey)
        }
    }

    @objc private func toggleCrashRestart() {
        CrashRestart.set(!CrashRestart.isEnabled)
    }

    @objc private func toggleLaunchAtLogin() {
        LaunchAtLogin.set(!LaunchAtLogin.isEnabled)
        if LaunchAtLogin.needsApproval {
            LaunchAtLogin.openSystemSettings()
        }
    }
}
