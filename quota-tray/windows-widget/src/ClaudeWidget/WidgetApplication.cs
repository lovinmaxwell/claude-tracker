using System;
using System.Linq;
using System.Windows;
using System.Windows.Threading;
using ClaudeUsageCore;
using Forms = System.Windows.Forms;

namespace ClaudeWidget;

/// <summary>Wires everything together: usage polling, the floating widget, the tray icon and its popover.</summary>
public sealed class WidgetApplication : Application
{
    private readonly UsageStore _store = new();
    private readonly AutoSessionRunner _autoSession = new();
    private readonly DispatcherTimer _clock = new() { Interval = TimeSpan.FromSeconds(15) };
    private readonly DispatcherTimer _saveDebounce = new() { Interval = TimeSpan.FromMilliseconds(400) };
    private WidgetWindow? _widget;
    private PopoverWindow? _popover;
    private Forms.NotifyIcon? _tray;
    private Forms.ContextMenuStrip? _menu;
    private Forms.ToolStripMenuItem? _toggleWidgetItem;
    private Forms.ToolStripMenuItem? _loginItem;
    private Forms.ToolStripMenuItem? _crashRestartItem;
    private Forms.ToolStripMenuItem? _autoSessionItem;
    private Forms.ToolStripMenuItem? _lastAutoSessionItem;

    public WidgetApplication()
    {
        // The widget lives in the tray; closing its window must not end the app.
        ShutdownMode = ShutdownMode.OnExplicitShutdown;

        // A stray exception on the UI thread is logged and survived instead of taking the widget down.
        DispatcherUnhandledException += (_, e) =>
        {
            AppLog.Write($"unhandled UI exception: {e.Exception}");
            e.Handled = AppLog.IsRecoverable(e.Exception);
        };
        AppDomain.CurrentDomain.UnhandledException += (_, e) => AppLog.Write($"fatal exception: {e.ExceptionObject}");
    }

    protected override void OnStartup(StartupEventArgs e)
    {
        base.OnStartup(e);
        var settings = SettingsStore.Current;

        var saved = settings is { AnchorX: { } x, AnchorY: { } y } ? new Point(x, y) : (Point?)null;
        _widget = new WidgetWindow(saved);
        _widget.DoubleClicked += ClaudeLauncher.OpenClaudeCode;
        _widget.ContextMenuRequested += () => _menu?.Show(Forms.Control.MousePosition);
        // A drag reports every pixel; write the position once it has settled.
        _saveDebounce.Tick += (_, _) =>
        {
            _saveDebounce.Stop();
            SettingsStore.Save();
        };
        _widget.AnchorChanged += anchor =>
        {
            settings.AnchorX = anchor.X;
            settings.AnchorY = anchor.Y;
            _saveDebounce.Stop();
            _saveDebounce.Start();
        };
        _popover = new PopoverWindow();

        BuildTray();
        Render();

        if (!settings.WidgetHidden)
        {
            _widget.Show();
        }

        _store.Updated += OnUsageUpdated;
        _clock.Tick += (_, _) => Render();
        _clock.Start();
        _store.Start();

        LaunchAtLogin.EnableOnFirstRun();
    }

    protected override void OnExit(ExitEventArgs e)
    {
        _clock.Stop();
        _store.Dispose();
        if (_tray is not null)
        {
            _tray.Visible = false;
            _tray.Icon?.Dispose();
            _tray.Dispose();
        }
        _menu?.Dispose();
        base.OnExit(e);
    }

    /// <summary>After each reading: redraw everything, then start the next session if the 5-hour window has ended.</summary>
    private async void OnUsageUpdated()
    {
        // async void: nothing may escape, or one bad redraw would crash the whole widget.
        try
        {
            Render();
            // A trigger starts a new window, so look at the usage again once it has run.
            // The cooldown stops that second reading from triggering another send.
            if (await _autoSession.StartIfNeededAsync(_store.Snapshot, readingFailed: _store.LastError is not null))
            {
                await _store.RefreshAsync();
            }
        }
        catch (Exception ex) when (AppLog.IsRecoverable(ex))
        {
            AppLog.Write($"error after a reading: {ex}");
        }
    }

    /// <summary>Redraws the widget, popover, tray icon and tooltip from the store.</summary>
    private void Render()
    {
        Theme.Refresh();
        var now = DateTimeOffset.UtcNow;
        var model = WidgetModel.From(_store.Snapshot, _store.LastError, now);
        _widget?.UpdateModel(model, now);
        _popover?.UpdateModel(model, now);

        if (_tray is not null)
        {
            var previous = _tray.Icon;
            _tray.Icon = TrayIconRenderer.Render(model.Peak?.Used, model.Status.Tone == Tone.Stale);
            previous?.Dispose();
            _tray.Text = Tooltip(model);
        }
    }

    /// <summary>"Claude · 5-hour 38% · 7-day 62%", trimmed to the 127 characters the tray allows.</summary>
    private static string Tooltip(WidgetModel model)
    {
        var parts = model.Rings.Select(r => $"{r.Label} {Formatters.Percent(r.Used)}");
        var text = model.Rings.Count > 0 ? $"Claude · {string.Join(" · ", parts)}" : $"Claude · {model.Status.Headline}";
        return text.Length > 127 ? text[..127] : text;
    }

    /// <summary>Creates the notification-area icon and its right-click menu.</summary>
    private void BuildTray()
    {
        _menu = new Forms.ContextMenuStrip();
        _menu.Items.Add("Open Claude Code", null, (_, _) => ClaudeLauncher.OpenClaudeCode());
        _menu.Items.Add("Open Claude App", null, (_, _) => ClaudeLauncher.OpenClaudeApp());
        _menu.Items.Add(new Forms.ToolStripSeparator());
        _menu.Items.Add("Refresh Now", null, async (_, _) => await _store.RefreshAsync());
        _toggleWidgetItem = new Forms.ToolStripMenuItem("Hide Widget", null, (_, _) => ToggleWidget());
        _menu.Items.Add(_toggleWidgetItem);
        _menu.Items.Add("Reset Position", null, (_, _) => _widget?.ResetPosition());
        _menu.Items.Add(new Forms.ToolStripSeparator());
        _loginItem = new Forms.ToolStripMenuItem("Launch at Login", null, (_, _) => LaunchAtLogin.Set(!LaunchAtLogin.IsEnabled));
        _menu.Items.Add(_loginItem);
        _crashRestartItem = new Forms.ToolStripMenuItem("Restart if It Crashes", null, (_, _) => CrashRestart.Set(!CrashRestart.IsEnabled));
        _menu.Items.Add(_crashRestartItem);
        _menu.Items.Add(new Forms.ToolStripSeparator());
        _autoSessionItem = new Forms.ToolStripMenuItem("Auto-Start New Session", null, async (_, _) => await ToggleAutoSession());
        _menu.Items.Add(_autoSessionItem);
        _menu.Items.Add("Set Trigger Message…", null, (_, _) => EditTriggerMessage());
        _menu.Items.Add("Send Trigger Now", null, async (_, _) =>
        {
            await _autoSession.RunAsync();
            await _store.RefreshAsync();
        });
        _lastAutoSessionItem = new Forms.ToolStripMenuItem("No auto-start yet") { Enabled = false };
        _menu.Items.Add(_lastAutoSessionItem);
        _menu.Items.Add(new Forms.ToolStripSeparator());
        _menu.Items.Add("Quit Claude Widget", null, (_, _) =>
        {
            // Remember the choice so the crash-restart task doesn't bring the widget straight back.
            SettingsStore.Current.UserQuit = true;
            SettingsStore.Save();
            Shutdown();
        });
        _menu.Opening += (_, _) => RefreshMenuState();

        _tray = new Forms.NotifyIcon
        {
            Icon = TrayIconRenderer.Render(null, isStale: false),
            Text = "Claude usage",
            ContextMenuStrip = _menu,
            Visible = true,
        };
        _tray.MouseUp += (_, e) =>
        {
            if (e.Button == Forms.MouseButtons.Left)
            {
                _ = _store.RefreshAsync();
                _popover?.ShowAtTray();
            }
        };
    }

    /// <summary>Brings the check marks and labels up to date each time the menu opens.</summary>
    private void RefreshMenuState()
    {
        var settings = SettingsStore.Current;
        if (_toggleWidgetItem is not null)
        {
            _toggleWidgetItem.Text = _widget?.IsVisible == true ? "Hide Widget" : "Show Widget";
        }
        if (_loginItem is not null)
        {
            _loginItem.Checked = LaunchAtLogin.IsEnabled;
        }
        if (_crashRestartItem is not null)
        {
            _crashRestartItem.Checked = CrashRestart.IsEnabled;
        }
        if (_autoSessionItem is not null)
        {
            _autoSessionItem.Checked = settings.AutoSessionEnabled;
        }
        if (_lastAutoSessionItem is not null)
        {
            _lastAutoSessionItem.Text = settings.LastAttempt is { } at
                ? $"Last auto-start: {Formatters.ResetClock(at, DateTimeOffset.UtcNow)} · {settings.LastResult}"
                : "No auto-start yet";
        }
    }

    /// <summary>Shows or hides the floating widget and remembers the choice.</summary>
    private void ToggleWidget()
    {
        if (_widget is null)
        {
            return;
        }
        var settings = SettingsStore.Current;
        if (_widget.IsVisible)
        {
            _widget.HideWidget();
            settings.WidgetHidden = true;
        }
        else
        {
            _widget.Show();
            settings.WidgetHidden = false;
        }
        SettingsStore.Save();
    }

    /// <summary>Turns auto-start on or off; turning it on checks straight away whether a session is due.</summary>
    private async System.Threading.Tasks.Task ToggleAutoSession()
    {
        var settings = SettingsStore.Current;
        settings.AutoSessionEnabled = !settings.AutoSessionEnabled;
        SettingsStore.Save();
        if (settings.AutoSessionEnabled
            && await _autoSession.StartIfNeededAsync(_store.Snapshot, readingFailed: _store.LastError is not null))
        {
            await _store.RefreshAsync();
        }
    }

    /// <summary>Asks for the message sent to start a new session.</summary>
    private static void EditTriggerMessage()
    {
        var entered = PromptWindow.Ask(
            "Trigger message",
            "Sent as claude -p \"…\" when your 5-hour session ends, to start the next one.",
            AutoSessionRunner.Message);
        if (entered is not null)
        {
            SettingsStore.Current.TriggerMessage = entered;
            SettingsStore.Save();
        }
    }
}
