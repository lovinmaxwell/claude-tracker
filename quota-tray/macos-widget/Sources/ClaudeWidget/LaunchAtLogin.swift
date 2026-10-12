import AppKit
import ServiceManagement

/// Registers the app as a login item so the widget is there after every restart.
enum LaunchAtLogin {
    private static let offeredKey = "didSetUpLaunchAtLogin"

    static var isEnabled: Bool {
        SMAppService.mainApp.status == .enabled
    }

    static var needsApproval: Bool {
        SMAppService.mainApp.status == .requiresApproval
    }

    static func set(_ enabled: Bool) {
        do {
            if enabled {
                try SMAppService.mainApp.register()
            } else {
                try SMAppService.mainApp.unregister()
            }
        } catch {
            NSLog("Claude Widget: launch at login failed: \(error.localizedDescription)")
        }
    }

    /// Turns launch-at-login on the first time the app runs; after that the
    /// menu item is the only thing that changes it.
    static func enableOnFirstRun() {
        let defaults = UserDefaults.standard
        guard !defaults.bool(forKey: offeredKey) else { return }
        defaults.set(true, forKey: offeredKey)
        set(true)
    }

    static func openSystemSettings() {
        SMAppService.openSystemSettingsLoginItems()
    }
}
