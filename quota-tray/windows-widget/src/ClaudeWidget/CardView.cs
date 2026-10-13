using System;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Media;
using System.Windows.Shapes;
using ClaudeUsageCore;

namespace ClaudeWidget;

/// <summary>The full widget: status, rings and reset times. Shared by the floating widget and the tray popover.</summary>
public sealed class CardView : Border
{
    public const double CardWidth = 336;
    public const double CardHeight = 204;

    private static readonly FontFamily Font = new("Segoe UI Variable Text, Segoe UI");

    private readonly Border _chip = new() { CornerRadius = new CornerRadius(11), Height = 22, HorizontalAlignment = HorizontalAlignment.Left, Padding = new Thickness(9, 0, 9, 0) };
    private readonly Ellipse _chipDot = new() { Width = 7, Height = 7, VerticalAlignment = VerticalAlignment.Center };
    private readonly TextBlock _chipText = Text(12, FontWeights.SemiBold);
    private readonly TextBlock _detail = Text(12, FontWeights.Normal);
    private readonly RingsControl _rings = new();
    private readonly StackPanel _rows = new() { VerticalAlignment = VerticalAlignment.Center };
    private readonly TextBlock _hint = Text(11, FontWeights.Normal);
    private readonly TextBlock _age = Text(11, FontWeights.Normal);

    public CardView()
    {
        Width = CardWidth;
        Height = CardHeight;
        CornerRadius = new CornerRadius(22);
        BorderThickness = new Thickness(1);
        Padding = new Thickness(16, 14, 16, 12);
        SnapsToDevicePixels = true;

        _rings.Configure(96, 9, 3, 0.1, shadow: false);
        _chipDot.Margin = new Thickness(0, 0, 6, 0);
        _chip.Child = new StackPanel { Orientation = Orientation.Horizontal, Children = { _chipDot, _chipText } };
        _detail.TextWrapping = TextWrapping.Wrap;
        _detail.Height = 32;
        _detail.Margin = new Thickness(0, 4, 0, 0);
        _age.HorizontalAlignment = HorizontalAlignment.Right;

        var body = new Grid { Margin = new Thickness(0, 8, 0, 0) };
        body.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });
        body.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
        _rings.VerticalAlignment = VerticalAlignment.Center;
        _rows.Margin = new Thickness(16, 0, 0, 0);
        Grid.SetColumn(_rows, 1);
        body.Children.Add(_rings);
        body.Children.Add(_rows);

        var footer = new Grid();
        footer.Children.Add(_hint);
        footer.Children.Add(_age);

        var layout = new Grid();
        layout.RowDefinitions.Add(new RowDefinition { Height = GridLength.Auto });
        layout.RowDefinitions.Add(new RowDefinition { Height = GridLength.Auto });
        layout.RowDefinitions.Add(new RowDefinition { Height = new GridLength(1, GridUnitType.Star) });
        layout.RowDefinitions.Add(new RowDefinition { Height = GridLength.Auto });
        Grid.SetRow(_detail, 1);
        Grid.SetRow(body, 2);
        Grid.SetRow(footer, 3);
        layout.Children.Add(_chip);
        layout.Children.Add(_detail);
        layout.Children.Add(body);
        layout.Children.Add(footer);
        Child = layout;
    }

    /// <summary>Redraws the card from the latest model; <paramref name="hint"/> is the footer's guidance line.</summary>
    public void Update(WidgetModel model, DateTimeOffset now, string hint)
    {
        var theme = Theme.Current;
        Background = Theme.Brush(theme.Card);
        BorderBrush = Theme.Brush(theme.Border);
        var ink = Theme.Brush(theme.Ink);
        var muted = Theme.Brush(theme.Muted);

        var tone = theme.ForTone(model.Status.Tone);
        _chip.Background = Theme.Brush(Color.FromArgb(0x24, tone.R, tone.G, tone.B));
        _chipDot.Fill = Theme.Brush(tone);
        _chipText.Foreground = Theme.Brush(tone);
        _chipText.Text = model.Status.Headline;

        _detail.Foreground = muted;
        _detail.Text = model.Status.Detail ?? " ";

        _rings.SetRings(model.Rings);

        _rows.Children.Clear();
        foreach (var ring in model.Rings)
        {
            _rows.Children.Add(BuildRow(ring, now, ink, muted));
        }
        if (model.Rings.Count == 0)
        {
            var empty = Text(11.5, FontWeights.Normal);
            empty.Text = "No usage reported yet.";
            empty.Foreground = muted;
            _rows.Children.Add(empty);
        }

        _hint.Foreground = muted;
        _hint.Text = hint;
        _age.Foreground = muted;
        _age.Text = model.Age is { } age ? $"Updated {age}" : string.Empty;
    }

    /// <summary>One ring's legend: swatch, label, percentage, and its reset time underneath.</summary>
    private static StackPanel BuildRow(RingModel ring, DateTimeOffset now, Brush ink, Brush muted)
    {
        var swatch = new Border
        {
            Width = 8,
            Height = 8,
            CornerRadius = new CornerRadius(3),
            Background = Theme.Brush(Theme.Resolve(ring.Color)),
            VerticalAlignment = VerticalAlignment.Center,
            Margin = new Thickness(0, 0, 7, 0),
        };
        var label = Text(12.5, FontWeights.Medium);
        label.Text = ring.Label;
        label.Foreground = muted;

        var percent = Text(15, FontWeights.Bold);
        percent.Text = Formatters.Percent(ring.Used);
        percent.Foreground = UsageAnalysis.ToneFor(ring.Used) == Tone.Hot ? Theme.Brush(Theme.Resolve(RingColor.Hot)) : ink;
        percent.HorizontalAlignment = HorizontalAlignment.Right;

        var head = new Grid();
        head.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });
        head.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
        head.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });
        Grid.SetColumn(label, 1);
        Grid.SetColumn(percent, 2);
        head.Children.Add(swatch);
        head.Children.Add(label);
        head.Children.Add(percent);

        var reset = Text(11, FontWeights.Normal);
        reset.Foreground = muted;
        reset.Margin = new Thickness(15, 1, 0, 0);
        reset.TextTrimming = TextTrimming.CharacterEllipsis;
        reset.Text = Formatters.Countdown(ring.ResetsAt, now) is { } countdown && Formatters.ResetClock(ring.ResetsAt, now) is { } clock
            ? $"Resets in {countdown} · {clock}"
            : "No reset time reported";

        return new StackPanel { Margin = new Thickness(0, 0, 0, 10), Children = { head, reset } };
    }

    /// <summary>A text block in the widget's typeface.</summary>
    private static TextBlock Text(double size, FontWeight weight) => new()
    {
        FontFamily = Font,
        FontSize = size,
        FontWeight = weight,
        TextTrimming = TextTrimming.CharacterEllipsis,
    };
}
