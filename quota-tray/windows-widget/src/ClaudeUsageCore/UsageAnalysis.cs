namespace ClaudeUsageCore;

public enum Tone
{
    Ok,
    Watch,
    Hot,
    Stale,
    Wait,
}

/// <summary>One plain-English sentence about where the user stands.</summary>
public sealed record UsageStatus(Tone Tone, string Headline, string? Detail);

/// <summary>Pace maths: how far through its window you are versus how much you've spent.</summary>
public static class UsageAnalysis
{
    /// <summary>Length of a known rolling window; null for ids we don't recognise.</summary>
    public static TimeSpan? WindowLength(string id) => id switch
    {
        "five_hour" => TimeSpan.FromHours(5),
        "seven_day" => TimeSpan.FromDays(7),
        _ => null,
    };

    /// <summary>Colour band for a usage figure; unknown usage is "wait", never "ok".</summary>
    public static Tone ToneFor(double? used) => used switch
    {
        null => Tone.Wait,
        >= 85 => Tone.Hot,
        >= 65 => Tone.Watch,
        _ => Tone.Ok,
    };

    /// <summary>Fraction (0..1) of the window already elapsed; null if length or reset time is unknown.</summary>
    public static double? ElapsedFraction(string windowId, DateTimeOffset? resetsAt, DateTimeOffset now)
    {
        if (WindowLength(windowId) is not { } length || resetsAt is not { } reset)
        {
            return null;
        }
        var remaining = reset - now;
        return remaining < TimeSpan.Zero || remaining > length ? null : 1 - remaining / length;
    }

    /// <summary>
    /// Hours until the limit at the pace seen so far; null when the reset arrives first or under
    /// 10% of the window has elapsed (too little signal to extrapolate).
    /// </summary>
    public static double? HoursToLimit(double used, string windowId, DateTimeOffset? resetsAt, DateTimeOffset now)
    {
        if (WindowLength(windowId) is not { } length
            || resetsAt is not { } reset
            || ElapsedFraction(windowId, resetsAt, now) is not { } elapsed
            || elapsed < 0.1
            || used <= 0)
        {
            return null;
        }
        var ratePerSecond = used / (elapsed * length.TotalSeconds);
        var secondsToLimit = (100 - used) / ratePerSecond;
        return secondsToLimit < (reset - now).TotalSeconds ? secondsToLimit / 3600 : null;
    }

    /// <summary>The window with the highest known usage.</summary>
    public static UsageWindow? Peak(IEnumerable<UsageWindow> windows) =>
        windows.Where(w => w.Used is not null).MaxBy(w => w.Used);

    /// <summary>One plain-English sentence for where the user stands; honest about stale or missing data.</summary>
    public static UsageStatus Summarize(UsageSnapshot? snapshot, bool isStale, UsageException? error, DateTimeOffset now)
    {
        if (snapshot is null || Peak(snapshot.Windows) is not { Used: { } used } peak)
        {
            return error is null
                ? new UsageStatus(Tone.Wait, "Waiting for a reading", null)
                : new UsageStatus(Tone.Stale, "Can't read usage", error.UserMessage);
        }

        if (isStale)
        {
            return new UsageStatus(
                Tone.Stale, error is null ? "Out of date" : "Can't refresh", "Showing the last reading");
        }

        var resetDetail = Formatters.Countdown(peak.ResetsAt, now) is { } countdown
            ? $"{peak.Label} resets in {countdown}"
            : null;

        if (used >= 100)
        {
            return new UsageStatus(Tone.Hot, "Limit reached", resetDetail);
        }
        if (HoursToLimit(used, peak.Id, peak.ResetsAt, now) is { } hours)
        {
            var whole = (int)hours;
            var minutes = (int)Math.Round((hours - whole) * 60);
            var eta = whole > 0 ? $"{whole}h {minutes}m" : $"{Math.Max(1, minutes)}m";
            return new UsageStatus(
                used >= 65 ? Tone.Hot : Tone.Watch,
                "Ahead of pace",
                $"{peak.Label} runs out in ~{eta} at this rate");
        }
        return used >= 85
            ? new UsageStatus(Tone.Hot, "Nearly out", resetDetail)
            : new UsageStatus(Tone.Ok, "On track", resetDetail);
    }
}
