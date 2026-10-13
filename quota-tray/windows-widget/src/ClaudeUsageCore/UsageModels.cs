namespace ClaudeUsageCore;

/// <summary>One rolling usage window as reported by Anthropic (5-hour session, 7-day week).</summary>
/// <param name="Used">0..100, or null when the vendor did not say. Never coerced to zero.</param>
public sealed record UsageWindow(string Id, string Label, double? Used, DateTimeOffset? ResetsAt);

/// <summary>A successful usage reading.</summary>
public sealed record UsageSnapshot(IReadOnlyList<UsageWindow> Windows, DateTimeOffset FetchedAt);

/// <summary>Everything that can stop a reading, each with a sentence fit for the widget.</summary>
public enum UsageErrorKind
{
    CredentialsMissing,
    CredentialsUnreadable,
    CredentialsMalformed,
    TokenExpired,
    Unauthorized,
    Http,
    Network,
    MalformedResponse,
}

/// <summary>A failed reading, carrying the kind of failure and a user-facing message.</summary>
public sealed class UsageException(UsageErrorKind kind, string? detail = null, int? httpStatus = null)
    : Exception(detail)
{
    public UsageErrorKind Kind { get; } = kind;

    public int? HttpStatus { get; } = httpStatus;

    /// <summary>A sentence the widget can show as-is.</summary>
    public string UserMessage => Kind switch
    {
        UsageErrorKind.CredentialsMissing => "Claude Code isn't signed in. Run `claude` once and log in.",
        UsageErrorKind.CredentialsUnreadable => "Couldn't read Claude Code's saved sign-in.",
        UsageErrorKind.CredentialsMalformed => "Couldn't read Claude Code's saved sign-in.",
        UsageErrorKind.TokenExpired => "Sign-in expired. Open Claude Code to refresh it.",
        UsageErrorKind.Unauthorized => "Anthropic rejected the sign-in. Open Claude Code to refresh it.",
        UsageErrorKind.Http => $"Anthropic returned HTTP {HttpStatus}.",
        UsageErrorKind.Network => "Can't reach Anthropic right now.",
        UsageErrorKind.MalformedResponse => "Anthropic sent a usage reply I couldn't read.",
        _ => "Something went wrong reading usage.",
    };
}
