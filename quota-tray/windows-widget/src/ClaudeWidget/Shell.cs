using System;
using System.ComponentModel;
using System.Diagnostics;
using System.IO;
using System.Security;
using System.Text;
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

/// <summary>
/// Optional supervision: a per-user scheduled task that relaunches the widget within five minutes if it
/// dies. It needs no admin rights (it runs as the current user, only while they are logged on). The widget
/// starts with <c>--watchdog</c> from the task and exits at once if it is already running, or if the user
/// chose Quit, so a deliberate Quit stays quit.
/// </summary>
public static class CrashRestart
{
    public const string LaunchArgument = "--watchdog";

    private const string TaskName = "Claude Widget Watchdog";

    /// <summary>True when the watchdog task exists.</summary>
    public static bool IsEnabled => RunSchtasks("/Query", "/TN", TaskName) == 0;

    /// <summary>Creates or removes the watchdog task.</summary>
    public static void Set(bool enabled)
    {
        if (!enabled)
        {
            RunSchtasks("/Delete", "/TN", TaskName, "/F");
            return;
        }
        if (Environment.ProcessPath is not { } exe)
        {
            return;
        }
        // The XML form lets the task run on battery power; the plain command-line form cannot.
        if (CreateFromXml(exe) != 0)
        {
            RunSchtasks("/Create", "/TN", TaskName, "/TR", $"\"{exe}\" {LaunchArgument}", "/SC", "MINUTE", "/MO", "5", "/F");
        }
    }

    /// <summary>Registers the task from a temporary XML definition; returns schtasks' exit code.</summary>
    private static int CreateFromXml(string exe)
    {
        var xml = $"""
            <?xml version="1.0" encoding="UTF-16"?>
            <Task version="1.2" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task">
              <Triggers>
                <TimeTrigger>
                  <Repetition>
                    <Interval>PT5M</Interval>
                    <StopAtDurationEnd>false</StopAtDurationEnd>
                  </Repetition>
                  <StartBoundary>2020-01-01T00:00:00</StartBoundary>
                  <Enabled>true</Enabled>
                </TimeTrigger>
              </Triggers>
              <Principals>
                <Principal id="Author">
                  <LogonType>InteractiveToken</LogonType>
                  <RunLevel>LeastPrivilege</RunLevel>
                </Principal>
              </Principals>
              <Settings>
                <MultipleInstancesPolicy>IgnoreNew</MultipleInstancesPolicy>
                <DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries>
                <StopIfGoingOnBatteries>false</StopIfGoingOnBatteries>
                <StartWhenAvailable>true</StartWhenAvailable>
                <ExecutionTimeLimit>PT0S</ExecutionTimeLimit>
                <Enabled>true</Enabled>
              </Settings>
              <Actions Context="Author">
                <Exec>
                  <Command>{SecurityElement.Escape(exe)}</Command>
                  <Arguments>{LaunchArgument}</Arguments>
                </Exec>
              </Actions>
            </Task>
            """;
        var file = Path.Combine(Path.GetTempPath(), $"claude-widget-watchdog-{Environment.ProcessId}.xml");
        try
        {
            // schtasks insists on UTF-16 for XML task files.
            File.WriteAllText(file, xml, Encoding.Unicode);
            return RunSchtasks("/Create", "/TN", TaskName, "/XML", file, "/F");
        }
        finally
        {
            File.Delete(file);
        }
    }

    /// <summary>Runs schtasks.exe with no window and returns its exit code (-1 if it could not run).</summary>
    private static int RunSchtasks(params string[] arguments)
    {
        var startInfo = new ProcessStartInfo("schtasks.exe")
        {
            UseShellExecute = false,
            CreateNoWindow = true,
            RedirectStandardOutput = true,
            RedirectStandardError = true,
        };
        foreach (var argument in arguments)
        {
            startInfo.ArgumentList.Add(argument);
        }
        try
        {
            using var process = Process.Start(startInfo);
            if (process is null)
            {
                return -1;
            }
            process.StandardOutput.ReadToEnd();
            process.StandardError.ReadToEnd();
            return process.WaitForExit(15_000) ? process.ExitCode : -1;
        }
        catch (Win32Exception)
        {
            return -1;
        }
    }
}
