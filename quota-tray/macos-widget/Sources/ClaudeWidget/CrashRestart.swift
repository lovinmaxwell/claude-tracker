import Foundation

/// Optional supervision: a per-user launchd agent that relaunches the widget if it crashes.
/// `SuccessfulExit: false` means a crash or kill is restarted, while "Quit" (exit status 0) stays quit.
/// No admin rights are needed; the agent lives in ~/Library/LaunchAgents and takes effect at the next login.
enum CrashRestart {
    private static let label = "dev.quotatray.claudewidget.keepalive"

    private static var plistURL: URL {
        FileManager.default.homeDirectoryForCurrentUser
            .appendingPathComponent("Library/LaunchAgents/\(label).plist")
    }

    static var isEnabled: Bool {
        FileManager.default.fileExists(atPath: plistURL.path)
    }

    static func set(_ enabled: Bool) {
        if enabled {
            install()
        } else {
            remove()
        }
    }

    /// Writes the agent's plist. launchd picks it up at the next login, so nothing is started now
    /// (which would race with the copy that is already running).
    private static func install() {
        // Only an installed .app can be relaunched by path; `swift run` has no bundle.
        guard let executable = Bundle.main.executablePath, Bundle.main.bundleIdentifier != nil else {
            NSLog("Claude Widget: restart-on-crash needs the installed app (run scripts/install.sh).")
            return
        }
        let plist: [String: Any] = [
            "Label": label,
            "ProgramArguments": [executable],
            "RunAtLoad": true,
            "KeepAlive": ["SuccessfulExit": false],
            "LimitLoadToSessionType": "Aqua",
            "ProcessType": "Interactive",
        ]
        do {
            let data = try PropertyListSerialization.data(fromPropertyList: plist, format: .xml, options: 0)
            try FileManager.default.createDirectory(
                at: plistURL.deletingLastPathComponent(), withIntermediateDirectories: true)
            try data.write(to: plistURL, options: .atomic)
        } catch {
            NSLog("Claude Widget: couldn't write the launch agent: \(error.localizedDescription)")
        }
    }

    /// Unloads the agent if it is loaded (harmless if not) and deletes its plist.
    private static func remove() {
        let process = Process()
        process.executableURL = URL(fileURLWithPath: "/bin/launchctl")
        process.arguments = ["bootout", "gui/\(getuid())/\(label)"]
        process.standardOutput = FileHandle.nullDevice
        process.standardError = FileHandle.nullDevice
        try? process.run()
        process.waitUntilExit()
        try? FileManager.default.removeItem(at: plistURL)
    }
}
