using System;
using System.Diagnostics;
using System.IO;
using System.Threading;
using System.Threading.Tasks;
using ClaudeUsageCore;

namespace ClaudeWidget;

/// <summary>
/// Keeps a Claude Code session rolling: once the 5-hour window has ended, runs
/// <c>claude -p "&lt;trigger message&gt;"</c> in the background so the next window starts. Opt-in,
/// because every trigger spends a little of the user's quota unattended.
/// </summary>
public sealed class AutoSessionRunner
{
    public const string DefaultMessage = "Reply with just the word: ok";

    private static readonly TimeSpan Timeout = TimeSpan.FromSeconds(180);

    private bool _isRunning;

    /// <summary>The message to send, or the default when none (or only whitespace) is saved.</summary>
    public static string Message =>
        string.IsNullOrWhiteSpace(SettingsStore.Current.TriggerMessage) ? DefaultMessage : SettingsStore.Current.TriggerMessage;

    /// <summary>
    /// Call after every usage reading; acts only if auto-start is on and the policy says it's due.
    /// Returns true if it sent a trigger, so the caller can look at the usage again.
    /// </summary>
    public async Task<bool> StartIfNeededAsync(UsageSnapshot? snapshot, bool readingFailed)
    {
        var settings = SettingsStore.Current;
        if (!settings.AutoSessionEnabled || _isRunning)
        {
            return false;
        }
        var due = AutoSessionPolicy.ShouldStart(
            snapshot, readingFailed, settings.LastAttempt, settings.LastAttemptSucceeded, DateTimeOffset.UtcNow);
        return due && await RunAsync();
    }

    /// <summary>Sends the trigger message now, whatever the window state ("Send Trigger Now"). False if one was already running.</summary>
    public async Task<bool> RunAsync()
    {
        if (_isRunning)
        {
            return false;
        }
        _isRunning = true;
        var settings = SettingsStore.Current;
        settings.LastAttempt = DateTimeOffset.UtcNow;
        settings.LastResult = "Running…";
        SettingsStore.Save();

        var (succeeded, summary) = await Task.Run(() => ExecuteAsync(Message));

        settings.LastAttemptSucceeded = succeeded;
        settings.LastResult = summary;
        SettingsStore.Save();
        _isRunning = false;
        return true;
    }

    /// <summary>
    /// Runs <c>claude -p</c> through PowerShell so PATH shims (.cmd, .ps1) resolve like in a terminal.
    /// The message travels in an environment variable, never spliced into the command line.
    /// </summary>
    private static async Task<(bool Succeeded, string Summary)> ExecuteAsync(string message)
    {
        var startInfo = new ProcessStartInfo("powershell.exe")
        {
            UseShellExecute = false,
            CreateNoWindow = true,
            RedirectStandardInput = true,
            RedirectStandardOutput = true,
            RedirectStandardError = true,
            WorkingDirectory = AppPaths.UserProfile,
        };
        foreach (var argument in new[]
        {
            "-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass",
            "-Command", "& claude -p $env:CLAUDE_WIDGET_MESSAGE",
        })
        {
            startInfo.ArgumentList.Add(argument);
        }
        startInfo.Environment["CLAUDE_WIDGET_MESSAGE"] = message;

        try
        {
            using var process = Process.Start(startInfo)
                ?? throw new InvalidOperationException("PowerShell did not start.");
            process.StandardInput.Close();
            var output = process.StandardOutput.ReadToEndAsync();
            var errors = process.StandardError.ReadToEndAsync();

            using var timeout = new CancellationTokenSource(Timeout);
            try
            {
                await process.WaitForExitAsync(timeout.Token);
            }
            catch (OperationCanceledException)
            {
                process.Kill(entireProcessTree: true);
                Log("timed out after 180s");
                return (false, "Timed out");
            }

            var text = (await output + await errors).Trim();
            Log($"exit {process.ExitCode}: {(text.Length > 300 ? text[..300] : text)}");
            return process.ExitCode switch
            {
                0 => (true, "OK"),
                _ when text.Contains("not recognized", StringComparison.OrdinalIgnoreCase) => (false, "`claude` not found"),
                var code => (false, $"Failed (exit {code})"),
            };
        }
        catch (Exception ex) when (ex is InvalidOperationException or System.ComponentModel.Win32Exception)
        {
            Log($"could not launch PowerShell: {ex.Message}");
            return (false, "Failed to launch");
        }
    }

    /// <summary>Appends a line to %LOCALAPPDATA%\ClaudeWidget\auto-session.log for working out why a trigger failed.</summary>
    private static void Log(string line)
    {
        try
        {
            Directory.CreateDirectory(AppPaths.LocalData);
            File.AppendAllText(
                Path.Combine(AppPaths.LocalData, "auto-session.log"),
                $"{DateTimeOffset.Now:O} {line}{Environment.NewLine}");
        }
        catch (Exception ex) when (ex is IOException or UnauthorizedAccessException)
        {
        }
    }
}
