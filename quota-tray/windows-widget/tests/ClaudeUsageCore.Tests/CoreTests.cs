using System.Globalization;
using System.Text;
using ClaudeUsageCore;
using Xunit;

namespace ClaudeUsageCore.Tests;

public sealed class UsageParserTests
{
    private static readonly DateTimeOffset Now = DateTimeOffset.FromUnixTimeSeconds(1_791_800_000);

    /// <summary>Encodes a JSON literal the way the HTTP client would hand it over.</summary>
    private static ReadOnlyMemory<byte> Utf8(string json) => Encoding.UTF8.GetBytes(json);

    [Fact]
    public void ParsesBothWindowsAndFractionalSeconds()
    {
        var snapshot = UsageParser.ParseSnapshot(Utf8("""
            {"five_hour":{"utilization":38.5,"resets_at":"2026-10-12T15:00:00.123456+00:00"},
             "seven_day":{"utilization":62,"resets_at":"2026-10-15T09:30:00Z"}}
            """), Now);

        Assert.Equal(["five_hour", "seven_day"], snapshot.Windows.Select(w => w.Id));
        Assert.Equal(38.5, snapshot.Windows[0].Used);
        Assert.Equal(62, snapshot.Windows[1].Used);
        Assert.Equal(
            DateTimeOffset.Parse("2026-10-12T15:00:00Z", CultureInfo.InvariantCulture).ToUnixTimeSeconds(),
            snapshot.Windows[0].ResetsAt!.Value.ToUnixTimeSeconds());
    }

    [Fact]
    public void ClampsAndKeepsNullAsUnknown()
    {
        var snapshot = UsageParser.ParseSnapshot(Utf8("""
            {"five_hour":{"utilization":140,"resets_at":null},"seven_day":{"utilization":null,"resets_at":null}}
            """), Now);

        Assert.Equal(100, snapshot.Windows[0].Used);
        Assert.Null(snapshot.Windows[1].Used);
        Assert.Null(snapshot.Windows[0].ResetsAt);
    }

    [Fact]
    public void HonoursTimezoneOffsets()
    {
        var date = UsageParser.ParseDate("2026-10-12T17:30:00.5+05:30");
        Assert.Equal(DateTimeOffset.Parse("2026-10-12T12:00:00.5Z", CultureInfo.InvariantCulture), date);
    }

    [Theory]
    [InlineData("nope")]
    [InlineData("{}")]
    [InlineData("[]")]
    public void RejectsGarbageAndEmpty(string json)
    {
        var error = Assert.Throws<UsageException>(() => UsageParser.ParseSnapshot(Utf8(json), Now));
        Assert.Equal(UsageErrorKind.MalformedResponse, error.Kind);
    }
}

public sealed class CredentialParserTests
{
    private static readonly DateTimeOffset Now = DateTimeOffset.FromUnixTimeSeconds(1_791_800_000);

    private static ReadOnlyMemory<byte> Utf8(string json) => Encoding.UTF8.GetBytes(json);

    /// <summary>The credentials JSON Claude Code writes, with the given expiry in epoch milliseconds.</summary>
    private static string OauthJson(long expiresAtMs) =>
        "{\"claudeAiOauth\":{\"accessToken\":\"tok\",\"expiresAt\":" + expiresAtMs + "}}";

    [Fact]
    public void ReadsTheTokenWhenNotExpired()
    {
        var future = (Now.ToUnixTimeSeconds() + 3600) * 1000;
        var json = OauthJson(future);
        Assert.Equal("tok", CredentialParser.ReadAccessToken(Utf8(json), Now));
    }

    [Fact]
    public void RejectsAnExpiredToken()
    {
        var past = (Now.ToUnixTimeSeconds() - 60) * 1000;
        var json = OauthJson(past);
        var error = Assert.Throws<UsageException>(() => CredentialParser.ReadAccessToken(Utf8(json), Now));
        Assert.Equal(UsageErrorKind.TokenExpired, error.Kind);
    }

    [Theory]
    [InlineData("{}")]
    [InlineData("""{"claudeAiOauth":{}}""")]
    [InlineData("""{"claudeAiOauth":{"accessToken":""}}""")]
    [InlineData("x")]
    public void RejectsMalformedCredentials(string json)
    {
        var error = Assert.Throws<UsageException>(() => CredentialParser.ReadAccessToken(Utf8(json), Now));
        Assert.Equal(UsageErrorKind.CredentialsMalformed, error.Kind);
    }
}

public sealed class AnalysisTests
{
    private static readonly DateTimeOffset Now = DateTimeOffset.FromUnixTimeSeconds(1_791_800_000);

    private static DateTimeOffset Hours(double hours) => Now.AddHours(hours);

    /// <summary>A snapshot with a 5-hour and a 7-day window at the given usage.</summary>
    private static UsageSnapshot Snapshot(double? five, double? seven, double fiveResetsIn = 4, double sevenResetsIn = 72) =>
        new(
            [
                new UsageWindow("five_hour", "5-hour", five, Hours(fiveResetsIn)),
                new UsageWindow("seven_day", "7-day", seven, Hours(sevenResetsIn)),
            ],
            Now);

    [Fact]
    public void FormatsCountdownsAndAges()
    {
        Assert.Equal("38m", Formatters.Countdown(Now.AddMinutes(38), Now));
        Assert.Equal("2h 14m", Formatters.Countdown(Now.AddMinutes(134), Now));
        Assert.Equal("3d 4h", Formatters.Countdown(Hours(3 * 24 + 4), Now));
        Assert.Equal("2d", Formatters.Countdown(Hours(48), Now));
        Assert.Equal("5h", Formatters.Countdown(Hours(5), Now));
        Assert.Null(Formatters.Countdown(Hours(-1), Now));
        Assert.Null(Formatters.Countdown(null, Now));
        Assert.Equal("just now", Formatters.Age(Now.AddSeconds(-3), Now));
        Assert.Equal("42s ago", Formatters.Age(Now.AddSeconds(-42), Now));
        Assert.Equal("5m ago", Formatters.Age(Now.AddSeconds(-300), Now));
    }

    [Fact]
    public void PercentNeverInventsZero()
    {
        Assert.Equal("—", Formatters.Percent(null));
        Assert.Equal("62%", Formatters.Percent(61.6));
    }

    [Fact]
    public void ElapsedFractionAndProjection()
    {
        Assert.Equal(0.8, UsageAnalysis.ElapsedFraction("five_hour", Hours(1), Now)!.Value, 9);
        Assert.Null(UsageAnalysis.ElapsedFraction("mystery", Hours(1), Now));
        // 60% after 4h of 5h: 40% left would take ~2.7h but only 1h remains, so the reset wins.
        Assert.Null(UsageAnalysis.HoursToLimit(60, "five_hour", Hours(1), Now));
        // 80% after 2h of 5h: 20% left at 40%/h is half an hour.
        Assert.Equal(0.5, UsageAnalysis.HoursToLimit(80, "five_hour", Hours(3), Now)!.Value, 9);
        // Under 10% elapsed: no extrapolation.
        Assert.Null(UsageAnalysis.HoursToLimit(30, "five_hour", Hours(4.8), Now));
    }

    [Fact]
    public void SummarisesWhereTheUserStands()
    {
        var calm = UsageAnalysis.Summarize(Snapshot(12, 40), false, null, Now);
        Assert.Equal(Tone.Ok, calm.Tone);
        Assert.Equal("On track", calm.Headline);
        Assert.Contains("resets in", calm.Detail);

        var fast = UsageAnalysis.Summarize(Snapshot(80, 10, fiveResetsIn: 3), false, null, Now);
        Assert.Equal("Ahead of pace", fast.Headline);
        Assert.Contains("runs out in ~30m", fast.Detail);

        Assert.Equal("Limit reached", UsageAnalysis.Summarize(Snapshot(100, 10), false, null, Now).Headline);
    }

    [Fact]
    public void IsHonestAboutStaleErroredAndMissingData()
    {
        var snapshot = Snapshot(12, 40);
        Assert.Equal("Out of date", UsageAnalysis.Summarize(snapshot, true, null, Now).Headline);
        var networkError = new UsageException(UsageErrorKind.Network);
        Assert.Equal("Can't refresh", UsageAnalysis.Summarize(snapshot, true, networkError, Now).Headline);

        var missing = new UsageException(UsageErrorKind.CredentialsMissing);
        var none = UsageAnalysis.Summarize(null, false, missing, Now);
        Assert.Equal("Can't read usage", none.Headline);
        Assert.Equal(missing.UserMessage, none.Detail);
        Assert.Equal(Tone.Wait, UsageAnalysis.Summarize(null, false, null, Now).Tone);
        Assert.Equal(Tone.Wait, UsageAnalysis.Summarize(Snapshot(null, null), false, null, Now).Tone);
    }

    [Fact]
    public void PeakIgnoresUnknown() =>
        Assert.Equal("seven_day", UsageAnalysis.Peak(Snapshot(null, 30).Windows)?.Id);
}

public sealed class AutoSessionPolicyTests
{
    private static readonly DateTimeOffset Now = DateTimeOffset.FromUnixTimeSeconds(1_791_800_000);

    /// <summary>A snapshot whose 5-hour window resets after the given hours (null: no reset time).</summary>
    private static UsageSnapshot Snapshot(double? fiveHourResetsIn, bool includeFiveHour = true, double ageSeconds = 5)
    {
        var windows = new List<UsageWindow> { new("seven_day", "7-day", 30, Now.AddDays(3)) };
        if (includeFiveHour)
        {
            windows.Insert(0, new UsageWindow("five_hour", "5-hour", 10, fiveHourResetsIn is { } h ? Now.AddHours(h) : null));
        }
        return new UsageSnapshot(windows, Now.AddSeconds(-ageSeconds));
    }

    private static bool Decide(UsageSnapshot? snapshot, bool failed = false, DateTimeOffset? last = null, bool lastOk = true) =>
        AutoSessionPolicy.ShouldStart(snapshot, failed, last, lastOk, Now);

    [Fact]
    public void DoesNothingWhileTheWindowIsRunning() => Assert.False(Decide(Snapshot(2)));

    [Fact]
    public void StartsWhenTheWindowHasEnded() => Assert.True(Decide(Snapshot(-0.1)));

    [Fact]
    public void StartsWhenThereIsNoResetTimeOrNoFiveHourWindow()
    {
        Assert.True(Decide(Snapshot(null)));
        Assert.True(Decide(Snapshot(null, includeFiveHour: false)));
    }

    [Fact]
    public void NeverActsOnMissingFailedOrStaleReadings()
    {
        Assert.False(Decide(null));
        Assert.False(Decide(Snapshot(-1), failed: true));
        Assert.False(Decide(Snapshot(-1, ageSeconds: 600)));
    }

    [Fact]
    public void WaitsAfterAnAttempt()
    {
        Assert.False(Decide(Snapshot(-1), last: Now.AddMinutes(-5)));
        Assert.True(Decide(Snapshot(-1), last: Now.AddMinutes(-16)));
    }

    [Fact]
    public void BacksOffLongerAfterAFailure()
    {
        Assert.False(Decide(Snapshot(-1), last: Now.AddMinutes(-30), lastOk: false));
        Assert.True(Decide(Snapshot(-1), last: Now.AddMinutes(-61), lastOk: false));
    }
}
