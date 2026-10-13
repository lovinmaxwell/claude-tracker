namespace ClaudeUsageCore;

/// <summary>Decides when to send the "trigger message" that starts the next 5-hour session.</summary>
public static class AutoSessionPolicy
{
    /// <summary>Minimum gap between attempts after one that worked.</summary>
    public static readonly TimeSpan Cooldown = TimeSpan.FromMinutes(15);

    /// <summary>Longer gap after a failed attempt (for instance `claude` isn't installed).</summary>
    public static readonly TimeSpan FailureCooldown = TimeSpan.FromHours(1);

    /// <summary>A reading older than this is too old to justify sending anything.</summary>
    public static readonly TimeSpan MaxReadingAge = TimeSpan.FromMinutes(3);

    /// <summary>
    /// A 5-hour window is running while its reset time is still in the future. A missing window, or one
    /// with no reset time or a reset in the past, means none is running.
    /// </summary>
    public static bool IsFiveHourWindowActive(UsageSnapshot snapshot, DateTimeOffset now) =>
        snapshot.Windows.FirstOrDefault(w => w.Id == "five_hour")?.ResetsAt is { } resetsAt && resetsAt > now;

    /// <summary>
    /// True only with a fresh, successful reading that shows no running window, and no recent attempt.
    /// Never guesses from stale, failed or missing data.
    /// </summary>
    public static bool ShouldStart(
        UsageSnapshot? snapshot,
        bool readingFailed,
        DateTimeOffset? lastAttempt,
        bool lastAttemptSucceeded,
        DateTimeOffset now)
    {
        if (snapshot is null || readingFailed || now - snapshot.FetchedAt >= MaxReadingAge)
        {
            return false;
        }
        if (lastAttempt is { } attempt
            && now - attempt < (lastAttemptSucceeded ? Cooldown : FailureCooldown))
        {
            return false;
        }
        return !IsFiveHourWindowActive(snapshot, now);
    }
}
