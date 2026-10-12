import AppKit

/// What a double-click on the widget opens.
enum ClaudeLauncher {
    enum Target: String {
        case terminal
        case desktopApp
    }

    private static let targetKey = "doubleClickTarget"

    static var preferredTarget: Target {
        get { Target(rawValue: UserDefaults.standard.string(forKey: targetKey) ?? "") ?? .terminal }
        set { UserDefaults.standard.set(newValue.rawValue, forKey: targetKey) }
    }

    static func open(_ target: Target) {
        switch target {
        case .terminal: openClaudeCodeInTerminal()
        case .desktopApp: openDesktopApp()
        }
    }

    /// Runs `claude` in a login shell inside Terminal. A `.command` file opens
    /// in Terminal with no Automation permission prompt, and `-il` loads the
    /// user's PATH so `claude` is found wherever it was installed.
    static func openClaudeCodeInTerminal() {
        let fileManager = FileManager.default
        let directory = fileManager.homeDirectoryForCurrentUser
            .appendingPathComponent("Library/Application Support/Claude Widget", isDirectory: true)
        let script = directory.appendingPathComponent("Claude Code.command")
        let body = "#!/bin/zsh -il\ncd \"$HOME\"\nclaude\n"
        do {
            try fileManager.createDirectory(at: directory, withIntermediateDirectories: true)
            try body.write(to: script, atomically: true, encoding: .utf8)
            try fileManager.setAttributes([.posixPermissions: 0o755], ofItemAtPath: script.path)
            NSWorkspace.shared.open(script)
        } catch {
            NSLog("Claude Widget: couldn't open Claude Code: \(error.localizedDescription)")
        }
    }

    static func openDesktopApp() {
        let process = Process()
        process.executableURL = URL(fileURLWithPath: "/usr/bin/open")
        process.arguments = ["-a", "Claude"]
        do {
            try process.run()
        } catch {
            NSLog("Claude Widget: couldn't open the Claude app: \(error.localizedDescription)")
        }
    }
}
