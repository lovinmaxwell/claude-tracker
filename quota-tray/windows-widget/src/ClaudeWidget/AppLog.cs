using System;
using System.IO;

namespace ClaudeWidget;

/// <summary>A small append-only log in %LOCALAPPDATA%\ClaudeWidget\widget.log, for working out why something failed.</summary>
public static class AppLog
{
    private const long MaxBytes = 1_000_000;

    private static readonly string FilePath = Path.Combine(AppPaths.LocalData, "widget.log");

    /// <summary>Appends a timestamped line; logging must never be the thing that fails, so I/O errors are ignored.</summary>
    public static void Write(string line)
    {
        try
        {
            Directory.CreateDirectory(AppPaths.LocalData);
            // Start over rather than grow forever.
            if (File.Exists(FilePath) && new FileInfo(FilePath).Length > MaxBytes)
            {
                File.Delete(FilePath);
            }
            File.AppendAllText(FilePath, $"{DateTimeOffset.Now:O} {line}{Environment.NewLine}");
        }
        catch (Exception ex) when (ex is IOException or UnauthorizedAccessException)
        {
        }
    }

    /// <summary>True for errors a supervisor can safely survive; out-of-memory and similar are left to crash.</summary>
    public static bool IsRecoverable(Exception ex) => ex is not (OutOfMemoryException or StackOverflowException or AccessViolationException);
}
