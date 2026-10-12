import Foundation

/// One rolling usage window as reported by Anthropic (5-hour session, 7-day week).
public struct UsageWindow: Equatable {
    public let id: String
    public let label: String
    /// 0...100, or nil when the vendor did not say. Never coerced to zero.
    public let used: Double?
    public let resetsAt: Date?

    public init(id: String, label: String, used: Double?, resetsAt: Date?) {
        self.id = id
        self.label = label
        self.used = used
        self.resetsAt = resetsAt
    }
}

public struct UsageSnapshot: Equatable {
    public let windows: [UsageWindow]
    public let fetchedAt: Date

    public init(windows: [UsageWindow], fetchedAt: Date) {
        self.windows = windows
        self.fetchedAt = fetchedAt
    }
}

/// Everything that can stop a reading, each with a sentence fit for the widget.
public enum UsageError: Error, Equatable {
    case credentialsMissing
    case credentialsUnreadable(String)
    case credentialsMalformed
    case tokenExpired
    case unauthorized
    case http(Int)
    case network(String)
    case malformedResponse

    public var userMessage: String {
        switch self {
        case .credentialsMissing:
            return "Claude Code isn't signed in. Run `claude` once and log in."
        case .credentialsUnreadable:
            return "macOS blocked Keychain access. Choose Always Allow when asked."
        case .credentialsMalformed:
            return "Couldn't read Claude Code's saved sign-in."
        case .tokenExpired:
            return "Sign-in expired. Open Claude Code to refresh it."
        case .unauthorized:
            return "Anthropic rejected the sign-in. Open Claude Code to refresh it."
        case .http(let status):
            return "Anthropic returned HTTP \(status)."
        case .network:
            return "Can't reach Anthropic right now."
        case .malformedResponse:
            return "Anthropic sent a usage reply I couldn't read."
        }
    }
}
