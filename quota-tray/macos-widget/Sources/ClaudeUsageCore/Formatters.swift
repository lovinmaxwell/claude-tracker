import Foundation

public enum Formatters {
    /// "2h 14m", "38m", "3d 4h"; nil once the moment has passed or when unknown.
    public static func countdown(to target: Date?, now: Date) -> String? {
        guard let target else { return nil }
        let totalMinutes = Int((target.timeIntervalSince(now) / 60).rounded(.down))
        if totalMinutes < 0 { return nil }
        if totalMinutes < 1 { return "<1m" }
        let days = totalMinutes / 1440
        let hours = (totalMinutes % 1440) / 60
        let minutes = totalMinutes % 60
        if days > 0 { return hours > 0 ? "\(days)d \(hours)h" : "\(days)d" }
        if hours > 0 { return minutes > 0 ? "\(hours)h \(minutes)m" : "\(hours)h" }
        return "\(minutes)m"
    }

    /// "8:11 PM" for today, "Thu 10:35 PM" for other days, in the user's locale.
    public static func resetClock(_ target: Date?, now: Date, calendar: Calendar = .current) -> String? {
        guard let target else { return nil }
        let formatter = DateFormatter()
        let sameDay = calendar.isDate(target, inSameDayAs: now)
        formatter.setLocalizedDateFormatFromTemplate(sameDay ? "jmm" : "EEEjmm")
        return formatter.string(from: target)
    }

    /// "just now", "42s ago", "5m ago".
    public static func age(of date: Date?, now: Date) -> String? {
        guard let date else { return nil }
        let seconds = max(0, Int(now.timeIntervalSince(date)))
        if seconds < 10 { return "just now" }
        if seconds < 60 { return "\(seconds)s ago" }
        return "\(seconds / 60)m ago"
    }

    /// "62%" or an em dash for unknown; never a fake zero.
    public static func percent(_ used: Double?) -> String {
        guard let used else { return "—" }
        return "\(Int(used.rounded()))%"
    }
}
