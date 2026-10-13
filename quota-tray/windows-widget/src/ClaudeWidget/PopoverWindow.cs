using System;
using System.Windows;
using System.Windows.Input;
using System.Windows.Media;
using Forms = System.Windows.Forms;

namespace ClaudeWidget;

/// <summary>The tray icon's popover: the very same card as the floating widget, closing when you click away.</summary>
public sealed class PopoverWindow : Window
{
    private const double EdgeMargin = 12;

    private readonly CardView _card = new();

    public PopoverWindow()
    {
        WindowStyle = WindowStyle.None;
        AllowsTransparency = true;
        Background = Brushes.Transparent;
        ResizeMode = ResizeMode.NoResize;
        ShowInTaskbar = false;
        Topmost = true;
        Title = "Claude usage";
        Width = CardView.CardWidth;
        Height = CardView.CardHeight;
        Content = _card;

        Deactivated += (_, _) => Hide();
        KeyDown += (_, e) =>
        {
            if (e.Key == Key.Escape)
            {
                Hide();
            }
        };
    }

    /// <summary>Redraws the card from the latest model.</summary>
    public void UpdateModel(WidgetModel model, DateTimeOffset now) =>
        _card.Update(model, now, "Right-click the tray icon for options");

    /// <summary>Opens the popover in the corner of the screen the pointer is on, just clear of the taskbar.</summary>
    public void ShowAtTray()
    {
        var dpi = VisualTreeHelper.GetDpi(this);
        var cursor = Forms.Cursor.Position;
        var area = Forms.Screen.FromPoint(cursor).WorkingArea;
        Left = (area.Right / dpi.DpiScaleX) - Width - EdgeMargin;
        Top = (area.Bottom / dpi.DpiScaleY) - Height - EdgeMargin;
        Show();
        Activate();
    }
}

/// <summary>A tiny modal text prompt, used for the trigger message.</summary>
public static class PromptWindow
{
    /// <summary>Asks for one line of text; returns null if the user cancels.</summary>
    public static string? Ask(string title, string explanation, string initial)
    {
        var input = new System.Windows.Controls.TextBox { Text = initial, Margin = new Thickness(0, 10, 0, 14), MinWidth = 340 };
        var save = new System.Windows.Controls.Button { Content = "Save", IsDefault = true, Width = 80, Margin = new Thickness(0, 0, 8, 0) };
        var cancel = new System.Windows.Controls.Button { Content = "Cancel", IsCancel = true, Width = 80 };
        var buttons = new System.Windows.Controls.StackPanel
        {
            Orientation = System.Windows.Controls.Orientation.Horizontal,
            HorizontalAlignment = HorizontalAlignment.Right,
            Children = { save, cancel },
        };
        var layout = new System.Windows.Controls.StackPanel
        {
            Margin = new Thickness(16),
            Children = { new System.Windows.Controls.TextBlock { Text = explanation, TextWrapping = TextWrapping.Wrap }, input, buttons },
        };
        var window = new Window
        {
            Title = title,
            Content = layout,
            SizeToContent = SizeToContent.WidthAndHeight,
            ResizeMode = ResizeMode.NoResize,
            WindowStartupLocation = WindowStartupLocation.CenterScreen,
            Topmost = true,
            ShowInTaskbar = false,
        };
        save.Click += (_, _) => window.DialogResult = true;
        window.Loaded += (_, _) =>
        {
            input.Focus();
            input.SelectAll();
        };
        return window.ShowDialog() == true ? input.Text : null;
    }
}
