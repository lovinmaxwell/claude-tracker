import Foundation

/// Decides when to send the "trigger message" that starts the next 5-hour session.
public enum AutoSessionPolicy {
    /// Minimum gap between attempts after one that worked.
    public static let cooldown: TimeInterval = 15 * 60
    /// Longer gap after a failed attempt (for instance `claude` isn't installed), so it doesn't retry in a tight loop.
    public static let failureCooldown: TimeInterval = 60 * 60
    /// A reading older than this is too old to justify sending anything.
    public static let maxReadingAge: TimeInterval = 3 * 60

    /// A 5-hour window is running while its reset time is still in the future.
    /// A missing window, or one with no reset time or a reset in the past, means none is running.
    public static func isFiveHourWindowActive(_ snapshot: UsageSnapshot, now: Date) -> Bool {
        guard
            let window = snapshot.windows.first(where: { $0.id == "five_hour" }),
            let resetsAt = window.resetsAt
        else { return false }
        return resetsAt > now
    }

    /// True only when we hold a fresh, successful reading that shows no running
    /// window, and we haven't just tried. Never guesses from stale or missing data.
    public static func shouldStart(
        snapshot: UsageSnapshot?,
        readingFailed: Bool,
        lastAttempt: Date?,
        lastAttemptSucceeded: Bool,
        now: Date
    ) -> Bool {
        guard let snapshot, !readingFailed else { return false }
        guard now.timeIntervalSince(snapshot.fetchedAt) < maxReadingAge else { return false }
        if let lastAttempt {
            let wait = lastAttemptSucceeded ? cooldown : failureCooldown
            if now.timeIntervalSince(lastAttempt) < wait { return false }
        }
        return !isFiveHourWindowActive(snapshot, now: now)
    }
}
