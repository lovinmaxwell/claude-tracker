import XCTest
@testable import ClaudeUsageCore

private let now = Date(timeIntervalSince1970: 1_791_800_000)

private func snapshot(fiveHourResetsIn hours: Double?, includeFiveHour: Bool = true, age: TimeInterval = 5) -> UsageSnapshot {
    var windows = [
        UsageWindow(id: "seven_day", label: "7-day", used: 30, resetsAt: now.addingTimeInterval(3 * 86_400)),
    ]
    if includeFiveHour {
        windows.insert(
            UsageWindow(
                id: "five_hour", label: "5-hour", used: 10,
                resetsAt: hours.map { now.addingTimeInterval($0 * 3600) }),
            at: 0)
    }
    return UsageSnapshot(windows: windows, fetchedAt: now.addingTimeInterval(-age))
}

private func decide(
    _ snap: UsageSnapshot?, failed: Bool = false, last: Date? = nil, lastOK: Bool = true
) -> Bool {
    AutoSessionPolicy.shouldStart(
        snapshot: snap, readingFailed: failed, lastAttempt: last, lastAttemptSucceeded: lastOK, now: now)
}

final class AutoSessionPolicyTests: XCTestCase {
    func testDoesNothingWhileTheWindowIsRunning() {
        XCTAssertFalse(decide(snapshot(fiveHourResetsIn: 2)))
    }

    func testStartsWhenTheWindowHasEnded() {
        XCTAssertTrue(decide(snapshot(fiveHourResetsIn: -0.1)))
    }

    func testStartsWhenThereIsNoResetTimeOrNoFiveHourWindow() {
        XCTAssertTrue(decide(snapshot(fiveHourResetsIn: nil)))
        XCTAssertTrue(decide(snapshot(fiveHourResetsIn: nil, includeFiveHour: false)))
    }

    func testNeverActsOnMissingFailedOrStaleReadings() {
        XCTAssertFalse(decide(nil))
        XCTAssertFalse(decide(snapshot(fiveHourResetsIn: -1), failed: true))
        XCTAssertFalse(decide(snapshot(fiveHourResetsIn: -1, age: 10 * 60)))
    }

    func testCooldownAfterAnAttempt() {
        let ended = snapshot(fiveHourResetsIn: -1)
        XCTAssertFalse(decide(ended, last: now.addingTimeInterval(-5 * 60)))
        XCTAssertTrue(decide(ended, last: now.addingTimeInterval(-16 * 60)))
    }

    func testBacksOffLongerAfterAFailure() {
        let ended = snapshot(fiveHourResetsIn: -1)
        XCTAssertFalse(decide(ended, last: now.addingTimeInterval(-30 * 60), lastOK: false))
        XCTAssertTrue(decide(ended, last: now.addingTimeInterval(-61 * 60), lastOK: false))
    }
}
