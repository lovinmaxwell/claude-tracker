import XCTest
@testable import ClaudeUsageCore

private let now = Date(timeIntervalSince1970: 1_791_800_000)
private func hours(_ h: Double) -> Date { now.addingTimeInterval(h * 3600) }

private func iso(_ date: Date) -> String {
    let f = ISO8601DateFormatter()
    f.formatOptions = [.withInternetDateTime]
    return f.string(from: date)
}

private func snapshot(five: Double?, seven: Double?, fiveResetsIn: Double = 4, sevenResetsIn: Double = 72) -> UsageSnapshot {
    UsageSnapshot(
        windows: [
            UsageWindow(id: "five_hour", label: "5-hour", used: five, resetsAt: hours(fiveResetsIn)),
            UsageWindow(id: "seven_day", label: "7-day", used: seven, resetsAt: hours(sevenResetsIn)),
        ],
        fetchedAt: now
    )
}

final class UsageParserTests: XCTestCase {
    func testParsesBothWindowsAndFractionalSeconds() throws {
        let json = """
        {"five_hour":{"utilization":38.5,"resets_at":"2026-10-12T15:00:00.123456+00:00"},
         "seven_day":{"utilization":62,"resets_at":"2026-10-15T09:30:00Z"}}
        """
        let snap = try UsageParser.snapshot(from: Data(json.utf8), fetchedAt: now)
        XCTAssertEqual(snap.windows.map(\.id), ["five_hour", "seven_day"])
        XCTAssertEqual(snap.windows[0].used, 38.5)
        XCTAssertEqual(snap.windows[1].used, 62)
        XCTAssertEqual(snap.windows[0].resetsAt, ISO8601DateFormatter().date(from: "2026-10-12T15:00:00Z"))
    }

    func testClampsAndKeepsNullAsUnknown() throws {
        let json = #"{"five_hour":{"utilization":140,"resets_at":null},"seven_day":{"utilization":null,"resets_at":null}}"#
        let snap = try UsageParser.snapshot(from: Data(json.utf8))
        XCTAssertEqual(snap.windows[0].used, 100)
        XCTAssertNil(snap.windows[1].used, "unknown must stay unknown, never zero")
        XCTAssertNil(snap.windows[0].resetsAt)
    }

    func testHonoursTimezoneOffsets() {
        let date = UsageParser.parseDate("2026-10-12T17:30:00.5+05:30")
        XCTAssertEqual(date, ISO8601DateFormatter().date(from: "2026-10-12T12:00:00Z"))
    }

    func testRejectsGarbageAndEmpty() {
        XCTAssertThrowsError(try UsageParser.snapshot(from: Data("nope".utf8)))
        XCTAssertThrowsError(try UsageParser.snapshot(from: Data("{}".utf8)))
    }
}

final class CredentialParserTests: XCTestCase {
    func testReadsTokenAndChecksExpiry() throws {
        let future = (now.timeIntervalSince1970 + 3600) * 1000
        let json = #"{"claudeAiOauth":{"accessToken":"tok","expiresAt":\#(Int(future))}}"#
        XCTAssertEqual(try CredentialParser.accessToken(from: Data(json.utf8), now: now), "tok")
    }

    func testExpiredToken() {
        let past = (now.timeIntervalSince1970 - 60) * 1000
        let json = #"{"claudeAiOauth":{"accessToken":"tok","expiresAt":\#(Int(past))}}"#
        XCTAssertThrowsError(try CredentialParser.accessToken(from: Data(json.utf8), now: now)) {
            XCTAssertEqual($0 as? UsageError, .tokenExpired)
        }
    }

    func testMalformed() {
        for bad in ["{}", #"{"claudeAiOauth":{}}"#, #"{"claudeAiOauth":{"accessToken":""}}"#, "x"] {
            XCTAssertThrowsError(try CredentialParser.accessToken(from: Data(bad.utf8), now: now)) {
                XCTAssertEqual($0 as? UsageError, .credentialsMalformed)
            }
        }
    }
}

final class AnalysisTests: XCTestCase {
    func testCountdownAndAge() {
        XCTAssertEqual(Formatters.countdown(to: now.addingTimeInterval(38 * 60), now: now), "38m")
        XCTAssertEqual(Formatters.countdown(to: now.addingTimeInterval((2 * 60 + 14) * 60), now: now), "2h 14m")
        XCTAssertEqual(Formatters.countdown(to: hours(3 * 24 + 4), now: now), "3d 4h")
        XCTAssertEqual(Formatters.countdown(to: hours(48), now: now), "2d")
        XCTAssertNil(Formatters.countdown(to: hours(-1), now: now))
        XCTAssertNil(Formatters.countdown(to: nil, now: now))
        XCTAssertEqual(Formatters.age(of: now.addingTimeInterval(-3), now: now), "just now")
        XCTAssertEqual(Formatters.age(of: now.addingTimeInterval(-42), now: now), "42s ago")
        XCTAssertEqual(Formatters.age(of: now.addingTimeInterval(-300), now: now), "5m ago")
    }

    func testPercentNeverInventsZero() {
        XCTAssertEqual(Formatters.percent(nil), "—")
        XCTAssertEqual(Formatters.percent(61.6), "62%")
    }

    func testElapsedFractionAndProjection() {
        XCTAssertEqual(UsageAnalysis.elapsedFraction(windowId: "five_hour", resetsAt: hours(1), now: now)!, 0.8, accuracy: 1e-9)
        XCTAssertNil(UsageAnalysis.elapsedFraction(windowId: "mystery", resetsAt: hours(1), now: now))
        // 60% after 4h of 5h: 40% left would take ~2.7h but only 1h remains, so the reset wins.
        XCTAssertNil(UsageAnalysis.hoursToLimit(used: 60, windowId: "five_hour", resetsAt: hours(1), now: now))
        // 80% after 2h of 5h: 20% left at 40%/h is half an hour.
        XCTAssertEqual(UsageAnalysis.hoursToLimit(used: 80, windowId: "five_hour", resetsAt: hours(3), now: now)!, 0.5, accuracy: 1e-9)
        // Under 10% elapsed: no extrapolation.
        XCTAssertNil(UsageAnalysis.hoursToLimit(used: 30, windowId: "five_hour", resetsAt: hours(4.8), now: now))
    }

    func testStatuses() {
        let calm = UsageAnalysis.summarize(snapshot: snapshot(five: 12, seven: 40), isStale: false, error: nil, now: now)
        XCTAssertEqual(calm.tone, .ok)
        XCTAssertEqual(calm.headline, "On track")
        XCTAssertTrue(calm.detail?.contains("resets in") == true)

        let fast = UsageAnalysis.summarize(
            snapshot: snapshot(five: 80, seven: 10, fiveResetsIn: 3), isStale: false, error: nil, now: now)
        XCTAssertEqual(fast.headline, "Ahead of pace")
        XCTAssertTrue(fast.detail?.contains("runs out in ~30m") == true)

        let full = UsageAnalysis.summarize(snapshot: snapshot(five: 100, seven: 10), isStale: false, error: nil, now: now)
        XCTAssertEqual(full.headline, "Limit reached")
    }

    func testHonestAboutStaleErroredAndMissingData() {
        let snap = snapshot(five: 12, seven: 40)
        XCTAssertEqual(UsageAnalysis.summarize(snapshot: snap, isStale: true, error: nil, now: now).headline, "Out of date")
        XCTAssertEqual(UsageAnalysis.summarize(snapshot: snap, isStale: true, error: .network("x"), now: now).headline, "Can't refresh")
        let none = UsageAnalysis.summarize(snapshot: nil, isStale: false, error: .credentialsMissing, now: now)
        XCTAssertEqual(none.headline, "Can't read usage")
        XCTAssertEqual(none.detail, UsageError.credentialsMissing.userMessage)
        XCTAssertEqual(UsageAnalysis.summarize(snapshot: nil, isStale: false, error: nil, now: now).tone, .wait)
        let unknown = snapshot(five: nil, seven: nil)
        XCTAssertEqual(UsageAnalysis.summarize(snapshot: unknown, isStale: false, error: nil, now: now).tone, .wait)
    }

    func testPeakIgnoresUnknown() {
        let snap = snapshot(five: nil, seven: 30)
        XCTAssertEqual(UsageAnalysis.peak(snap.windows)?.id, "seven_day")
    }
}
