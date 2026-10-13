using System;
using System.ComponentModel;
using System.Diagnostics;
using Microsoft.Win32;

namespace ClaudeWidget;

/// <summary>Registers the exe as a per-user login item (HKCU Run), which needs no admin rights.</summary>
public static class LaunchAtLogin
{
    private const string RunKeyPath = @"Software\Microsoft\Windows\CurrentVersion\Run";
    private const string ValueName = "Claude Widget";

    private static string Command => $"\"{Environment.ProcessPath}\"";

    /// <summary>True when the login item points at this exact exe.</summary>
    public static bool IsEnabled
    {
        get
        {
            using var key = Registry.CurrentUser.OpenSubKey(RunKeyPath);
            return key?.GetValue(ValueName) as string == Command;
        }
    }

    /// <summary>Adds or removes the login item.</summary>
    public static void Set(bool enabled)
    {
        using var key = Registry.CurrentUser.CreateSubKey(RunKeyPath);
        if (enabled)
        {
            key.SetValue(ValueName, Command);
        }
        else
        {
            key.DeleteValue(ValueName, throwOnMissingValue: false);
        }
    }

    /// <summary>Turns launch-at-login on the first time the app runs; after that the menu item is in charge.</summary>
    public static void EnableOnFirstRun()
    {
        var settings = SettingsStore.Current;
        if (settings.LaunchAtLoginOffered || Environment.ProcessPath is null)
        {
            return;
        }
        settings.LaunchAtLoginOffered = true;
        SettingsStore.Save();
        Set(true);
    }
}

/// <summary>What a double-click on the widget opens.</summary>
public static class ClaudeLauncher
{
    /// <summary>Runs <c>claude</c> in a terminal window: Windows Terminal if present, otherwise cmd.</summary>
    public static void OpenClaudeCode()
    {
        try
        {
            var terminal = new ProcessStartInfo("wt.exe") { UseShellExecute = false, WorkingDirectory = AppPaths.UserProfile };
            foreach (var argument in new[] { "-d", AppPaths.UserProfile, "cmd.exe", "/k", "claude" })
            {
                terminal.ArgumentList.Add(argument);
            }
            Process.Start(terminal);
        }
        catch (Win32Exception)
        {
            var cmd = new ProcessStartInfo("cmd.exe", "/k claude")
            {
                UseShellExecute = true,
                WorkingDirectory = AppPaths.UserProfile,
            };
            Process.Start(cmd);
        }
    }

    /// <summary>Opens the Claude desktop app through its URL scheme, if it is installed.</summary>
    public static void OpenClaudeApp()
    {
        try
        {
            Process.Start(new ProcessStartInfo("claude://") { UseShellExecute = true });
        }
        catch (Win32Exception)
        {
        }
    }
}
