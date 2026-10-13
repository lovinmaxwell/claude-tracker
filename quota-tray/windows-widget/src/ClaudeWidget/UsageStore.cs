using System;
using System.IO;
using System.Net;
using System.Net.Http;
using System.Net.Http.Headers;
using System.Threading.Tasks;
using System.Windows.Threading;
using ClaudeUsageCore;

namespace ClaudeWidget;

/// <summary>Reads Claude Code's saved sign-in from %USERPROFILE%\.claude\.credentials.json.</summary>
public static class CredentialReader
{
    private static readonly string FilePath = Path.Combine(AppPaths.UserProfile, ".claude", ".credentials.json");

    /// <summary>Returns the access token, or throws a <see cref="UsageException"/> saying why it can't.</summary>
    public static async Task<string> ReadAccessTokenAsync()
    {
        if (!File.Exists(FilePath))
        {
            throw new UsageException(UsageErrorKind.CredentialsMissing);
        }
        try
        {
            var bytes = await File.ReadAllBytesAsync(FilePath);
            return CredentialParser.ReadAccessToken(bytes, DateTimeOffset.UtcNow);
        }
        catch (Exception ex) when (ex is IOException or UnauthorizedAccessException)
        {
            throw new UsageException(UsageErrorKind.CredentialsUnreadable, ex.Message);
        }
    }
}

/// <summary>
/// Polls Anthropic once a minute. A failed poll keeps the last good reading (marked stale)
/// instead of flashing a confident zero.
/// </summary>
public sealed class UsageStore : IDisposable
{
    private static readonly Uri Endpoint = new("https://api.anthropic.com/api/oauth/usage");

    /// <summary>A poll still running after this long is abandoned so it can't block every later one.</summary>
    private static readonly TimeSpan HungPollLimit = TimeSpan.FromSeconds(45);

    private readonly HttpClient _http = new() { Timeout = TimeSpan.FromSeconds(20) };
    private readonly DispatcherTimer _timer = new() { Interval = TimeSpan.FromSeconds(60) };
    private bool _inFlight;

    public UsageSnapshot? Snapshot { get; private set; }

    public UsageException? LastError { get; private set; }

    /// <summary>Raised on the UI thread after every poll, successful or not.</summary>
    public event Action? Updated;

    /// <summary>Takes a first reading now and then one every minute.</summary>
    public void Start()
    {
        _timer.Tick += async (_, _) => await RefreshAsync();
        _timer.Start();
        _ = RefreshAsync();
    }

    /// <summary>
    /// Polls once; overlapping calls collapse into one. This is the supervised boundary: whatever goes wrong
    /// inside a poll becomes a "can't refresh" state with the last good reading kept, never a crash or a stall.
    /// </summary>
    public async Task RefreshAsync()
    {
        if (_inFlight)
        {
            return;
        }
        _inFlight = true;
        try
        {
            Snapshot = await FetchAsync().WaitAsync(HungPollLimit);
            LastError = null;
        }
        catch (UsageException error)
        {
            LastError = error;
        }
        catch (TimeoutException)
        {
            LastError = new UsageException(UsageErrorKind.TimedOut);
        }
        catch (Exception ex) when (AppLog.IsRecoverable(ex))
        {
            AppLog.Write($"unexpected error while polling: {ex}");
            LastError = new UsageException(UsageErrorKind.Unexpected, ex.Message);
        }
        finally
        {
            _inFlight = false;
        }
        Updated?.Invoke();
    }

    /// <summary>One authenticated request to the usage endpoint.</summary>
    private async Task<UsageSnapshot> FetchAsync()
    {
        var token = await CredentialReader.ReadAccessTokenAsync();
        using var request = new HttpRequestMessage(HttpMethod.Get, Endpoint);
        request.Headers.Authorization = new AuthenticationHeaderValue("Bearer", token);
        request.Headers.Add("anthropic-beta", "oauth-2025-04-20");
        request.Headers.Accept.Add(new MediaTypeWithQualityHeaderValue("application/json"));

        try
        {
            using var response = await _http.SendAsync(request);
            var body = await response.Content.ReadAsByteArrayAsync();
            return response.StatusCode switch
            {
                _ when response.IsSuccessStatusCode => UsageParser.ParseSnapshot(body, DateTimeOffset.UtcNow),
                HttpStatusCode.Unauthorized or HttpStatusCode.Forbidden => throw new UsageException(UsageErrorKind.Unauthorized),
                _ => throw new UsageException(UsageErrorKind.Http, httpStatus: (int)response.StatusCode),
            };
        }
        catch (Exception ex) when (ex is HttpRequestException or TaskCanceledException)
        {
            throw new UsageException(UsageErrorKind.Network, ex.Message);
        }
    }

    public void Dispose()
    {
        _timer.Stop();
        _http.Dispose();
    }
}
