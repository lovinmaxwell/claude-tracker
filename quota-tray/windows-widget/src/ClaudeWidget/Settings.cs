using System;
using System.IO;
using System.Text.Json;

namespace ClaudeWidget;

/// <summary>Everything the widget remembers between runs, stored in %APPDATA%\ClaudeWidget\settings.json.</summary>
public sealed class AppSettings
{
    /// <summary>Top-left corner of the resting rings, in device-independent pixels.</summary>
    public double? AnchorX { get; set; }

    public double? AnchorY { get; set; }

    public bool WidgetHidden { get; set; }

    public bool LaunchAtLoginOffered { get; set; }

    public bool AutoSessionEnabled { get; set; }

    public string? TriggerMessage { get; set; }

    public DateTimeOffset? LastAttempt { get; set; }

    public bool LastAttemptSucceeded { get; set; } = true;

    public string? LastResult { get; set; }
}

/// <summary>Loads and saves <see cref="AppSettings"/>; a missing or corrupt file just means defaults.</summary>
public static class SettingsStore
{
    private static readonly JsonSerializerOptions Options = new() { WriteIndented = true };

    private static readonly string FilePath = Path.Combine(AppPaths.RoamingData, "settings.json");

    /// <summary>The live settings object shared by the whole app.</summary>
    public static AppSettings Current { get; } = Load();

    /// <summary>Writes the current settings to disk; failures are ignored because settings are a convenience.</summary>
    public static void Save()
    {
        try
        {
            Directory.CreateDirectory(AppPaths.RoamingData);
            File.WriteAllText(FilePath, JsonSerializer.Serialize(Current, Options));
        }
        catch (IOException)
        {
        }
        catch (UnauthorizedAccessException)
        {
        }
    }

    /// <summary>Reads the settings file, falling back to defaults.</summary>
    private static AppSettings Load()
    {
        try
        {
            return File.Exists(FilePath)
                ? JsonSerializer.Deserialize<AppSettings>(File.ReadAllText(FilePath)) ?? new AppSettings()
                : new AppSettings();
        }
        catch (Exception ex) when (ex is IOException or JsonException or UnauthorizedAccessException)
        {
            return new AppSettings();
        }
    }
}

/// <summary>Per-user folders; nothing outside the user's profile is ever touched.</summary>
public static class AppPaths
{
    public static string RoamingData { get; } =
        Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), "ClaudeWidget");

    public static string LocalData { get; } =
        Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "ClaudeWidget");

    public static string UserProfile { get; } = Environment.GetFolderPath(Environment.SpecialFolder.UserProfile);
}
