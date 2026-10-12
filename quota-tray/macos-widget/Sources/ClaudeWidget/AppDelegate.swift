import AppKit
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
