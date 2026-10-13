import Foundation
import ClaudeUsageCore

/// Keeps a Claude Code session rolling: once the 5-hour window has ended, runs
/// `claude -p "<trigger message>"` in the background so the next window starts.
/// Opt-in, because every trigger spends a little of the user's quota unattended.
final class AutoSessionRunner {
    static let defaultMessage = "Reply with just the word: ok"

    private static let enabledKey = "autoSessionEnabled"
    private static let messageKey = "autoSessionMessage"
    private static let lastAttemptKey = "autoSessionLastAttempt"
    private static let lastOKKey = "autoSessionLastOK"
    private static let lastResultKey = "autoSessionLastResult"

    private static let timeout: TimeInterval = 180
    private var isRunning = false

    static var isEnabled: Bool {
        get { UserDefaults.standard.bool(forKey: enabledKey) }
        set { UserDefaults.standard.set(newValue, forKey: enabledKey) }
    }

    static var message: String {
        get {
            let stored = UserDefaults.standard.string(forKey: messageKey) ?? ""
            return stored.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty ? defaultMessage : stored
        }
        set { UserDefaults.standard.set(newValue, forKey: messageKey) }
    }

    static var lastAttempt: Date? { UserDefaults.standard.object(forKey: lastAttemptKey) as? Date }
    static var lastResult: String? { UserDefaults.standard.string(forKey: lastResultKey) }
    private static var lastAttemptSucceeded: Bool {
        UserDefaults.standard.object(forKey: lastOKKey) as? Bool ?? true
    }

    /// Call after every usage reading. Does nothing unless auto-start is on and the
    /// policy says the 5-hour window has ended with no new one running.
    func startIfNeeded(snapshot: UsageSnapshot?, readingFailed: Bool, now: Date, completion: @escaping () -> Void) {
        guard AutoSessionRunner.isEnabled, !isRunning else { return }
        let due = AutoSessionPolicy.shouldStart(
            snapshot: snapshot,
            readingFailed: readingFailed,
            lastAttempt: AutoSessionRunner.lastAttempt,
            lastAttemptSucceeded: AutoSessionRunner.lastAttemptSucceeded,
            now: now
        )
        if due { run(completion: completion) }
    }

    /// Sends the trigger message now, whatever the window state ("Send Trigger Now").
    func run(completion: @escaping () -> Void) {
        guard !isRunning else { return }
        isRunning = true
        let defaults = UserDefaults.standard
        defaults.set(Date(), forKey: AutoSessionRunner.lastAttemptKey)
        defaults.set("Running…", forKey: AutoSessionRunner.lastResultKey)

        DispatchQueue.global(qos: .utility).async {
            let message = AutoSessionRunner.message
            let outcome = AutoSessionRunner.execute(message: message)
            DispatchQueue.main.async {
                defaults.set(outcome.succeeded, forKey: AutoSessionRunner.lastOKKey)
                defaults.set(outcome.summary, forKey: AutoSessionRunner.lastResultKey)
                self.isRunning = false
                completion()
            }
        }
    }

    private struct Outcome {
        let succeeded: Bool
        let summary: String
    }

    /// Runs `claude -p` in an interactive login shell so PATH matches the user's
    /// terminal (nvm, ~/.local/bin, Homebrew…). The message travels in an
    /// environment variable, never spliced into the shell command.
    private static func execute(message: String) -> Outcome {
        let process = Process()
        process.executableURL = URL(fileURLWithPath: "/bin/zsh")
        process.arguments = ["-ilc", "claude -p \"$CLAUDE_WIDGET_MESSAGE\""]
        var environment = ProcessInfo.processInfo.environment
        environment["CLAUDE_WIDGET_MESSAGE"] = message
        process.environment = environment
        process.standardInput = FileHandle.nullDevice
        let output = Pipe()
        process.standardOutput = output
        process.standardError = output

        do {
            try process.run()
        } catch {
            log("could not launch zsh: \(error.localizedDescription)")
            return Outcome(succeeded: false, summary: "Failed to launch")
        }
        DispatchQueue.global().asyncAfter(deadline: .now() + timeout) {
            if process.isRunning { process.terminate() }
        }
        let data = output.fileHandleForReading.readDataToEndOfFile()
        process.waitUntilExit()

        let text = String(decoding: data, as: UTF8.self).trimmingCharacters(in: .whitespacesAndNewlines)
        let status = process.terminationStatus
        log("exit \(status): \(text.prefix(300))")
        if status == 0 {
            return Outcome(succeeded: true, summary: "OK")
        }
        return Outcome(succeeded: false, summary: status == 127 ? "`claude` not found" : "Failed (exit \(status))")
    }

    /// ~/Library/Logs/Claude Widget/auto-session.log, for working out why a trigger failed.
    private static func log(_ line: String) {
        let directory = FileManager.default.homeDirectoryForCurrentUser
            .appendingPathComponent("Library/Logs/Claude Widget", isDirectory: true)
        let file = directory.appendingPathComponent("auto-session.log")
        let entry = "\(ISO8601DateFormatter().string(from: Date())) \(line)\n"
        do {
            try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
            if let handle = try? FileHandle(forWritingTo: file) {
                defer { try? handle.close() }
                try handle.seekToEnd()
                try handle.write(contentsOf: Data(entry.utf8))
            } else {
                try entry.write(to: file, atomically: true, encoding: .utf8)
            }
        } catch {
            NSLog("Claude Widget: couldn't write log: \(error.localizedDescription)")
        }
    }
}
