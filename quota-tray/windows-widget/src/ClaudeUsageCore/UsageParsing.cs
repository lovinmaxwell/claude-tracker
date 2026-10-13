using System.Globalization;
using System.Text.Json;

namespace ClaudeUsageCore;

/// <summary>Parses the <c>GET /api/oauth/usage</c> reply.</summary>
public static class UsageParser
{
    private static readonly (string Key, string Label)[] Known =
    [
        ("five_hour", "5-hour"),
        ("seven_day", "7-day"),
    ];

    /// <summary>Turns the JSON body into a snapshot, or throws <see cref="UsageException"/> if it isn't one.</summary>
    public static UsageSnapshot ParseSnapshot(ReadOnlyMemory<byte> utf8Json, DateTimeOffset fetchedAt)
    {
        try
        {
            using var document = JsonDocument.Parse(utf8Json);
            var root = document.RootElement;
            if (root.ValueKind != JsonValueKind.Object)
            {
                throw new UsageException(UsageErrorKind.MalformedResponse);
            }

            var windows = new List<UsageWindow>(Known.Length);
            foreach (var (key, label) in Known)
            {
                if (!root.TryGetProperty(key, out var window) || window.ValueKind != JsonValueKind.Object)
                {
                    continue;
                }
                windows.Add(new UsageWindow(key, label, ReadUsed(window), ReadResetTime(window)));
            }

            return windows.Count > 0
                ? new UsageSnapshot(windows, fetchedAt)
                : throw new UsageException(UsageErrorKind.MalformedResponse);
        }
        catch (JsonException)
        {
            throw new UsageException(UsageErrorKind.MalformedResponse);
        }
    }

    /// <summary>RFC 3339 with or without fractional seconds ("2026-10-12T15:00:00.123456+00:00").</summary>
    public static DateTimeOffset? ParseDate(string? raw) =>
        DateTimeOffset.TryParse(raw, CultureInfo.InvariantCulture, DateTimeStyles.None, out var parsed)
            ? parsed
            : null;

    /// <summary>Utilisation clamped to 0..100; null stays null so unknown is never shown as zero.</summary>
    private static double? ReadUsed(JsonElement window) =>
        window.TryGetProperty("utilization", out var value) && value.ValueKind == JsonValueKind.Number
            ? Math.Clamp(value.GetDouble(), 0, 100)
            : null;

    /// <summary>The window's reset moment, if the reply carries a readable one.</summary>
    private static DateTimeOffset? ReadResetTime(JsonElement window) =>
        window.TryGetProperty("resets_at", out var value) && value.ValueKind == JsonValueKind.String
            ? ParseDate(value.GetString())
            : null;
}

/// <summary>Pulls the OAuth access token out of the JSON Claude Code keeps on disk.</summary>
public static class CredentialParser
{
    /// <summary>Returns the access token, or throws if it is missing, malformed or expired.</summary>
    public static string ReadAccessToken(ReadOnlyMemory<byte> utf8Json, DateTimeOffset now)
    {
        try
        {
            using var document = JsonDocument.Parse(utf8Json);
            if (!document.RootElement.TryGetProperty("claudeAiOauth", out var oauth)
                || oauth.ValueKind != JsonValueKind.Object
                || !oauth.TryGetProperty("accessToken", out var tokenElement)
                || tokenElement.ValueKind != JsonValueKind.String
                || tokenElement.GetString() is not { Length: > 0 } token)
            {
                throw new UsageException(UsageErrorKind.CredentialsMalformed);
            }

            // Claude Code stores the expiry in milliseconds since the epoch.
            if (oauth.TryGetProperty("expiresAt", out var expires)
                && expires.ValueKind == JsonValueKind.Number
                && expires.GetDouble() / 1000 <= now.ToUnixTimeSeconds())
            {
                throw new UsageException(UsageErrorKind.TokenExpired);
            }

            return token;
        }
        catch (JsonException)
        {
            throw new UsageException(UsageErrorKind.CredentialsMalformed);
        }
    }
}
