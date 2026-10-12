import Foundation

private func number(_ value: Any?) -> Double? {
    if let d = value as? Double { return d }
    if let i = value as? Int { return Double(i) }
    return nil
}

/// Parses the `GET /api/oauth/usage` reply.
public enum UsageParser {
    private static let known: [(key: String, label: String)] = [
        ("five_hour", "5-hour"),
        ("seven_day", "7-day"),
    ]

    public static func snapshot(from data: Data, fetchedAt: Date = Date()) throws -> UsageSnapshot {
        guard let root = (try? JSONSerialization.jsonObject(with: data)) as? [String: Any] else {
            throw UsageError.malformedResponse
        }
        var windows: [UsageWindow] = []
        for (key, label) in known {
            guard let dict = root[key] as? [String: Any] else { continue }
            let used = number(dict["utilization"]).map { min(100, max(0, $0)) }
            windows.append(
                UsageWindow(
                    id: key,
                    label: label,
                    used: used,
                    resetsAt: parseDate(dict["resets_at"] as? String)
                )
            )
        }
        guard !windows.isEmpty else { throw UsageError.malformedResponse }
        return UsageSnapshot(windows: windows, fetchedAt: fetchedAt)
    }

    /// RFC 3339 with or without fractional seconds ("2026-10-12T15:00:00.123456+00:00").
    public static func parseDate(_ raw: String?) -> Date? {
        guard var text = raw else { return nil }
        if let dot = text.firstIndex(of: ".") {
            let afterDot = text[text.index(after: dot)...]
            if let end = afterDot.firstIndex(where: { !$0.isNumber }) {
                text = String(text[..<dot]) + String(text[end...])
            } else {
                text = String(text[..<dot]) + "Z"
            }
        }
        let formatter = ISO8601DateFormatter()
        formatter.formatOptions = [.withInternetDateTime]
        return formatter.date(from: text)
    }
}

/// Pulls the OAuth access token out of the JSON Claude Code keeps in the Keychain.
public enum CredentialParser {
    public static func accessToken(from data: Data, now: Date = Date()) throws -> String {
        guard
            let root = (try? JSONSerialization.jsonObject(with: data)) as? [String: Any],
            let oauth = root["claudeAiOauth"] as? [String: Any],
            let token = oauth["accessToken"] as? String,
            !token.isEmpty
        else {
            throw UsageError.credentialsMalformed
        }
        // Claude Code stores the expiry in milliseconds since the epoch.
        if let expiresAtMs = number(oauth["expiresAt"]), expiresAtMs / 1000 <= now.timeIntervalSince1970 {
            throw UsageError.tokenExpired
        }
        return token
    }
}
