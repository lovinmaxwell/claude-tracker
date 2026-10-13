using System;
using System.Runtime.InteropServices;
using System.Windows;
using System.Windows.Media.Animation;
using System.Windows.Controls;
using System.Windows.Input;
using System.Windows.Interop;
using System.Windows.Media;
using System.Windows.Threading;
using Forms = System.Windows.Forms;

namespace ClaudeWidget;

/// <summary>
/// The floating widget: a borderless, always-on-top window that rests as faded rings and grows into
/// the full card while the pointer is over it. Drag with the left button, double-click to open Claude Code.
/// </summary>
public sealed class WidgetWindow : Window
{
    private const double RestSize = 100;
    private const double RestingOpacity = 0.6;
    private const double ScreenMargin = 12;
    private const double DefaultInset = 16;

    private const int GwlExStyle = -20;
    private const int WsExToolWindow = 0x00000080;
    private const int WsExNoActivate = 0x08000000;

    private readonly RingsControl _rings = new();
    private readonly CardView _card = new();
    private readonly DispatcherTimer _collapseTimer = new() { Interval = TimeSpan.FromMilliseconds(250) };
    private Point _anchor;
    private Point _shift;
    private bool _expanded;
    private bool _programmatic;

    public event Action? DoubleClicked;

    public event Action? ContextMenuRequested;

    /// <summary>Raised with the new top-left anchor whenever the user drags the widget.</summary>
    public event Action<Point>? AnchorChanged;

    public WidgetWindow(Point? savedAnchor)
    {
        WindowStyle = WindowStyle.None;
        AllowsTransparency = true;
        Background = Brushes.Transparent;
        ResizeMode = ResizeMode.NoResize;
        ShowInTaskbar = false;
        ShowActivated = false;
        Topmost = true;
        Title = "Claude Widget";
        Width = Height = RestSize;
        Opacity = RestingOpacity;

        _rings.Configure(96, 9, 3, 0.28, shadow: true);
        _rings.HorizontalAlignment = HorizontalAlignment.Left;
        _rings.VerticalAlignment = VerticalAlignment.Top;
        _rings.Margin = new Thickness(2);
        _card.HorizontalAlignment = HorizontalAlignment.Left;
        _card.VerticalAlignment = VerticalAlignment.Top;
        _card.Visibility = Visibility.Collapsed;

        var root = new Grid();
        root.Children.Add(_rings);
        root.Children.Add(_card);
        Content = root;

        _anchor = savedAnchor is { } saved && IsOnScreen(saved) ? saved : DefaultAnchor();
        (Left, Top) = (_anchor.X, _anchor.Y);

        SourceInitialized += (_, _) => MakeNonActivating();
        MouseEnter += (_, _) =>
        {
            _collapseTimer.Stop();
            SetExpanded(true);
        };
        MouseLeave += (_, _) =>
        {
            _collapseTimer.Stop();
            _collapseTimer.Start();
        };
        _collapseTimer.Tick += (_, _) =>
        {
            _collapseTimer.Stop();
            // Trust where the pointer actually is, not a stray leave event during the resize.
            if (!IsMouseOver)
            {
                SetExpanded(false);
            }
        };
        MouseLeftButtonDown += OnLeftButtonDown;
        MouseRightButtonUp += (_, _) => ContextMenuRequested?.Invoke();
        LocationChanged += OnLocationChanged;
    }

    /// <summary>Redraws both layers from the latest model.</summary>
    public void UpdateModel(WidgetModel model, DateTimeOffset now)
    {
        _rings.SetRings(model.Rings);
        _card.Update(model, now, "Double-click opens Claude Code");
    }

    /// <summary>Hides the widget, folding the card back first so it reappears as rings.</summary>
    public void HideWidget()
    {
        _collapseTimer.Stop();
        SetExpanded(false);
        Hide();
    }

    /// <summary>Puts the widget back in the top-right corner of the primary screen.</summary>
    public void ResetPosition()
    {
        SetExpanded(false);
        _anchor = DefaultAnchor();
        _programmatic = true;
        (Left, Top) = (_anchor.X, _anchor.Y);
        _programmatic = false;
        AnchorChanged?.Invoke(_anchor);
    }

    /// <summary>Left button: double-click opens Claude Code, a press-and-move drags the window.</summary>
    private void OnLeftButtonDown(object sender, MouseButtonEventArgs e)
    {
        if (e.ClickCount >= 2)
        {
            DoubleClicked?.Invoke();
            return;
        }
        try
        {
            DragMove();
        }
        catch (InvalidOperationException)
        {
            // The button was already released; nothing to drag.
        }
    }

    /// <summary>Remembers where the user dropped the widget, undoing any on-screen shift of the card.</summary>
    private void OnLocationChanged(object? sender, EventArgs e)
    {
        if (_programmatic || !IsLoaded)
        {
            return;
        }
        _anchor = new Point(Left + _shift.X, Top + _shift.Y);
        AnchorChanged?.Invoke(_anchor);
    }

    /// <summary>Swaps between the resting rings and the full card, keeping the card on screen.</summary>
    private void SetExpanded(bool expanded)
    {
        if (_expanded == expanded)
        {
            return;
        }
        _expanded = expanded;
        var (width, height) = expanded ? (CardView.CardWidth, CardView.CardHeight) : (RestSize, RestSize);
        _shift = expanded ? ShiftToFit(width, height) : default;

        _programmatic = true;
        try
        {
            Width = width;
            Height = height;
            Left = _anchor.X - _shift.X;
            Top = _anchor.Y - _shift.Y;
        }
        finally
        {
            _programmatic = false;
        }

        _rings.Visibility = expanded ? Visibility.Collapsed : Visibility.Visible;
        _card.Visibility = expanded ? Visibility.Visible : Visibility.Collapsed;
        BeginAnimation(OpacityProperty, new DoubleAnimation(expanded ? 1 : RestingOpacity, TimeSpan.FromMilliseconds(200)));
    }

    /// <summary>How far left and up the card must slide so it stays inside its screen's usable area.</summary>
    private Point ShiftToFit(double width, double height)
    {
        var area = WorkAreaAt(_anchor);
        var overflowX = Math.Max(0, _anchor.X + width - (area.Right - ScreenMargin));
        var roomX = Math.Max(0, _anchor.X - (area.Left + ScreenMargin));
        var overflowY = Math.Max(0, _anchor.Y + height - (area.Bottom - ScreenMargin));
        var roomY = Math.Max(0, _anchor.Y - (area.Top + ScreenMargin));
        return new Point(Math.Min(overflowX, roomX), Math.Min(overflowY, roomY));
    }

    /// <summary>The usable (taskbar-free) area of the screen holding a point, in device-independent pixels.</summary>
    private Rect WorkAreaAt(Point point)
    {
        var dpi = VisualTreeHelper.GetDpi(this);
        var screen = Forms.Screen.FromPoint(new System.Drawing.Point((int)(point.X * dpi.DpiScaleX), (int)(point.Y * dpi.DpiScaleY)));
        var area = screen.WorkingArea;
        return new Rect(area.X / dpi.DpiScaleX, area.Y / dpi.DpiScaleY, area.Width / dpi.DpiScaleX, area.Height / dpi.DpiScaleY);
    }

    /// <summary>A saved anchor is only trusted if it still lands on a connected display.</summary>
    private static bool IsOnScreen(Point point) =>
        new Rect(SystemParameters.VirtualScreenLeft, SystemParameters.VirtualScreenTop, SystemParameters.VirtualScreenWidth, SystemParameters.VirtualScreenHeight)
            .Contains(point);

    /// <summary>First run: top-right corner of the primary screen's usable area.</summary>
    private static Point DefaultAnchor()
    {
        var area = SystemParameters.WorkArea;
        return new Point(area.Right - RestSize - DefaultInset, area.Top + DefaultInset);
    }

    /// <summary>Stops clicks from stealing focus and keeps the widget out of Alt-Tab.</summary>
    private void MakeNonActivating()
    {
        var handle = new WindowInteropHelper(this).Handle;
        _ = SetWindowLong(handle, GwlExStyle, GetWindowLong(handle, GwlExStyle) | WsExNoActivate | WsExToolWindow);
    }

    [DllImport("user32.dll", EntryPoint = "GetWindowLongW")]
    private static extern int GetWindowLong(IntPtr hWnd, int nIndex);

    [DllImport("user32.dll", EntryPoint = "SetWindowLongW")]
    private static extern int SetWindowLong(IntPtr hWnd, int nIndex, int dwNewLong);
}
