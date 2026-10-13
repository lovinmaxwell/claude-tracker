import Foundation

/// The Claude Tracker icon has 12 pills around its ring and 12 body rows on the creature; both light
/// up together as usage grows. This is the one place that turns a percentage into that count.
public enum IconFill {
    public static let steps = 12

    /// How many of the 12 steps are lit; unknown (nil) lights none, never a made-up reading.
    public static func litSteps(_ percent: Double?) -> Int {
        Int((min(max(percent ?? 0, 0), 100) / 100 * Double(steps)).rounded())
    }
}
