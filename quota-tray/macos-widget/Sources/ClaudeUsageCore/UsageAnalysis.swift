import Foundation

public enum Tone: String, Equatable {
    case ok, watch, hot, stale, wait
}

public struct UsageStatus: Equatable {
    public let tone: Tone
    public let headline: String
    public let detail: String?

    public init(tone: Tone, headline: String, detail: String?) {
        self.tone = tone
        self.headline = headline
        self.detail = detail
    }
}

/// Pace maths: how far through its window you are versus how much you've spent.
public enum UsageAnalysis {
    private static let windowLength: [String: TimeInterval] = [
        "five_hour": 5 * 3600,
        "seven_day": 7 * 24 * 3600,
    ]

    public static func tone(forUsed used: Double?) -> Tone {
        guard let used else { return .wait }
        if used >= 85 { return .hot }
        if used >= 65 { return .watch }
        return .ok
    }

    /// Fraction (0...1) of the window already elapsed; nil if length or reset time is unknown.
    public static func elapsedFraction(windowId: String, resetsAt: Date?, now: Date) -> Double? {
        guard let length = windowLength[windowId], let resetsAt else { return nil }
        let remaining = resetsAt.timeIntervalSince(now)
        guard remaining >= 0, remaining <= length else { return nil }
        return 1 - remaining / length
    }

    /// Hours until the limit at the pace seen so far; nil when the reset arrives first
    /// or under 10% of the window has elapsed (too little signal to extrapolate).
    public static func hoursToLimit(used: Double, windowId: String, resetsAt: Date?, now: Date) -> Double? {
        guard
            let length = windowLength[windowId],
            let resetsAt,
            let elapsed = elapsedFraction(windowId: windowId, resetsAt: resetsAt, now: now),
            elapsed >= 0.1, used > 0
        else { return nil }
        let ratePerSecond = used / (elapsed * length)
        let secondsToLimit = (100 - used) / ratePerSecond
        return secondsToLimit < resetsAt.timeIntervalSince(now) ? secondsToLimit / 3600 : nil
    }

    /// The window with the highest known usage.
    public static func peak(_ windows: [UsageWindow]) -> UsageWindow? {
        windows
            .filter { $0.used != nil }
            .max { ($0.used ?? 0) < ($1.used ?? 0) }
    }

    /// One plain-English sentence for where the user stands.
    public static func summarize(
        snapshot: UsageSnapshot?,
        isStale: Bool,
        error: UsageError?,
        now: Date
    ) -> UsageStatus {
        guard let snapshot, let peak = peak(snapshot.windows), let used = peak.used else {
            if let error {
                return UsageStatus(tone: .stale, headline: "Can't read usage", detail: error.userMessage)
            }
            return UsageStatus(tone: .wait, headline: "Waiting for a reading", detail: nil)
        }
        if isStale {
            return UsageStatus(
                tone: .stale,
                headline: error != nil ? "Can't refresh" : "Out of date",
                detail: "Showing the last reading"
            )
        }

        let reset = Formatters.countdown(to: peak.resetsAt, now: now)
        let resetDetail = reset.map { "\(peak.label) resets in \($0)" }

        if used >= 100 {
            return UsageStatus(tone: .hot, headline: "Limit reached", detail: resetDetail)
        }
        if let hours = hoursToLimit(used: used, windowId: peak.id, resetsAt: peak.resetsAt, now: now) {
            let whole = Int(hours)
            let minutes = Int(((hours - Double(whole)) * 60).rounded())
            let eta = whole > 0 ? "\(whole)h \(minutes)m" : "\(max(1, minutes))m"
            return UsageStatus(
                tone: used >= 65 ? .hot : .watch,
                headline: "Ahead of pace",
                detail: "\(peak.label) runs out in ~\(eta) at this rate"
            )
        }
        if used >= 85 {
            return UsageStatus(tone: .hot, headline: "Nearly out", detail: resetDetail)
        }
        return UsageStatus(tone: .ok, headline: "On track", detail: resetDetail)
    }
}
