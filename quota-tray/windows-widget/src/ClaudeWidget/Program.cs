using System;
using System.Linq;
using System.Threading;
using ClaudeWidget;

// One widget per user session: a second launch (for example the login item plus a double-click) just exits.
using var singleInstance = new Mutex(initiallyOwned: true, @"Local\ClaudeWidget.SingleInstance", out var isFirstInstance);
if (!isFirstInstance)
{
    return;
}

// A launch from the crash-restart task passes --watchdog. If the user chose Quit, stay quit; a launch the user
// starts themselves (double-click, login item) clears that choice.
var settings = SettingsStore.Current;
var launchedByWatchdog = args.Contains(CrashRestart.LaunchArgument);
if (launchedByWatchdog && settings.UserQuit)
{
    return;
}
if (!launchedByWatchdog && settings.UserQuit)
{
    settings.UserQuit = false;
    SettingsStore.Save();
}

// WPF needs a single-threaded apartment, which top-level statements can't request directly.
var uiThread = new Thread(() => new WidgetApplication().Run()) { Name = "UI" };
uiThread.SetApartmentState(ApartmentState.STA);
uiThread.Start();
uiThread.Join();
