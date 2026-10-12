import AppKit
import Combine
import ClaudeUsageCore

final class AppDelegate: NSObject, NSApplicationDelegate, NSMenuDelegate {
    private enum Tag: Int {
        case toggleWidget = 1
        case doubleClickTerminal = 10
        case doubleClickApp = 11
        case launchAtLogin = 20
    }

    private let store = UsageStore()
    private var controller: WidgetPanelController?
    private var statusItem: NSStatusItem?
    private var storeObserver: AnyCancellable?
    /// Menu items with tags in this range are the live usage rows at the top of the menu.
    private static let usageTags = 30..<40
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
        if let image = NSImage(systemSymbolName: "gauge", accessibilityDescription: "Claude usage") {
            item.button?.image = image
        } else {
            item.button?.title = "Claude"
        }
        item.menu = makeMenu()
        statusItem = item
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

    /// Rebuilds the greyed-out usage rows at the top of the menu each time it opens.
    private func refreshUsageRows(in menu: NSMenu) {
        for existing in menu.items where AppDelegate.usageTags.contains(existing.tag) {
            menu.removeItem(existing)
        }
        let rows = usageRows()
        for (index, title) in rows.enumerated() {
            let row = NSMenuItem(title: title, action: nil, keyEquivalent: "")
            row.tag = AppDelegate.usageTags.lowerBound + index
            row.isEnabled = false
            menu.insertItem(row, at: index)
        }
        let divider = NSMenuItem.separator()
        divider.tag = AppDelegate.usageTags.upperBound - 1
        menu.insertItem(divider, at: rows.count)
    }

    private func updateTooltip() {
        statusItem?.button?.toolTip = usageRows().joined(separator: "\n")
    }

    func menuNeedsUpdate(_ menu: NSMenu) {
        refreshUsageRows(in: menu)
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
            case nil:
                break
            }
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

    @objc private func toggleLaunchAtLogin() {
        LaunchAtLogin.set(!LaunchAtLogin.isEnabled)
        if LaunchAtLogin.needsApproval {
            LaunchAtLogin.openSystemSettings()
        }
    }
}
