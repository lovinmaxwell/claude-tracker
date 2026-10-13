using System.Globalization;

namespace ClaudeUsageCore;

/// <summary>Short human-readable strings for countdowns, clock times and ages.</summary>
public static class Formatters
{
    /// <summary>"2h 14m", "38m", "3d 4h"; null once the moment has passed or when unknown.</summary>
    public static string? Countdown(DateTimeOffset? target, DateTimeOffset now)
    {
        if (target is not { } moment)
        {
            return null;
        }
        var totalMinutes = (int)Math.Floor((moment - now).TotalMinutes);
        if (totalMinutes < 0)
        {
            return null;
        }
        var (days, hours, minutes) = (totalMinutes / 1440, totalMinutes % 1440 / 60, totalMinutes % 60);
        return (days, hours, minutes, totalMinutes) switch
        {
            (_, _, _, < 1) => "<1m",
            ( > 0, > 0, _, _) => $"{days}d {hours}h",
            ( > 0, _, _, _) => $"{days}d",
            (_, > 0, > 0, _) => $"{hours}h {minutes}m",
            (_, > 0, _, _) => $"{hours}h",
            _ => $"{minutes}m",
        };
    }

    /// <summary>"8:11 PM" for today, "Thu 10:35 PM" for other days, in the given culture (local time).</summary>
    public static string? ResetClock(DateTimeOffset? target, DateTimeOffset now, CultureInfo? culture = null)
    {
        if (target is not { } moment)
        {
            return null;
        }
        culture ??= CultureInfo.CurrentCulture;
        var local = moment.ToLocalTime();
        var time = local.ToString("t", culture);
        return local.Date == now.ToLocalTime().Date ? time : $"{local.ToString("ddd", culture)} {time}";
    }

    /// <summary>"just now", "42s ago", "5m ago".</summary>
    public static string? Age(DateTimeOffset? at, DateTimeOffset now)
    {
        if (at is not { } moment)
        {
            return null;
        }
        var seconds = Math.Max(0, (int)(now - moment).TotalSeconds);
        return seconds switch
        {
            < 10 => "just now",
            < 60 => $"{seconds}s ago",
            _ => $"{seconds / 60}m ago",
        };
    }

    /// <summary>"62%" or an em dash for unknown; never a fake zero.</summary>
    public static string Percent(double? used) =>
        used is { } value ? $"{(int)Math.Round(value, MidpointRounding.AwayFromZero)}%" : "—";
}
