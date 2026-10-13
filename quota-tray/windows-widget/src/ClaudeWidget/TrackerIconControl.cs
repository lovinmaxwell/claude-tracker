using System;
using System.Windows;
using System.Windows.Media;
using System.Windows.Media.Effects;
using ClaudeUsageCore;

namespace ClaudeWidget;

/// <summary>
/// The Claude Tracker icon: a ring of 12 rounded pills around the creature. The pills fill clockwise from the
/// top as usage grows (the last quarter turns coral as a warning) while the creature fills from its feet up,
/// exactly as in the original menu bar icon. Used as the resting state of the floating widget.
/// </summary>
public sealed class TrackerIconControl : FrameworkElement
{
    private const int PillCount = 12;
    private const double Size = 96;

    private static readonly Brush HitBackdrop = Theme.Brush(Color.FromArgb(1, 0, 0, 0));
    private static readonly Brush EmptyPill = Theme.Brush(Color.FromArgb(115, Mascot.Unspent.R, Mascot.Unspent.G, Mascot.Unspent.B));
    private static readonly Brush SpentPill = Theme.Brush(Mascot.Spent);
    private static readonly Brush WarningPill = Theme.Brush(Mascot.Warning);
    private static readonly Brush StalePill = Theme.Brush(Color.FromArgb(140, Mascot.Spent.R, Mascot.Spent.G, Mascot.Spent.B));
    private static readonly Brush StaleWarningPill = Theme.Brush(Color.FromArgb(140, Mascot.Warning.R, Mascot.Warning.G, Mascot.Warning.B));

    private double? _used;
    private bool _isStale;

    public TrackerIconControl()
    {
        // A soft shadow keeps the icon readable on light and dark wallpapers alike.
        Effect = new DropShadowEffect { BlurRadius = 4, ShadowDepth = 1, Opacity = 0.4, Color = Colors.Black };
    }

    /// <summary>Sets the peak usage (null when unknown) and whether it is out of date, then redraws.</summary>
    public void SetUsage(double? used, bool isStale)
    {
        (_used, _isStale) = (used, isStale);
        InvalidateVisual();
    }

    protected override Size MeasureOverride(Size availableSize) => new(Size, Size);

    protected override void OnRender(DrawingContext drawingContext)
    {
        var center = new Point(Size / 2, Size / 2);

        // Nearly invisible, but it makes the whole disc hit-testable so hovering the gaps still counts.
        drawingContext.DrawEllipse(HitBackdrop, null, center, Size / 2, Size / 2);

        Mascot.Draw(drawingContext, center, Size * 0.75 / 16, _used);
        DrawPills(drawingContext, center);
    }

    /// <summary>Twelve short arcs around the edge; the proportions match the original 32-pixel icon, scaled.</summary>
    private void DrawPills(DrawingContext context, Point center)
    {
        var scale = Size / 32;
        var radius = 13.4 * scale;
        var thickness = 2 * 1.35 * scale;
        var filled = IconFill.LitSteps(_used);
        var step = 2 * Math.PI / PillCount;

        for (var index = 0; index < PillCount; index++)
        {
            var middle = -Math.PI / 2 + (index + 0.5) * step;
            var start = middle - 0.10;
            var end = middle + 0.10;

            Brush brush = EmptyPill;
            if (index < filled)
            {
                var warning = index >= 9;
                brush = _isStale ? (warning ? StaleWarningPill : StalePill) : (warning ? WarningPill : SpentPill);
            }

            var pen = new Pen(brush, thickness) { StartLineCap = PenLineCap.Round, EndLineCap = PenLineCap.Round };
            pen.Freeze();

            var geometry = new StreamGeometry();
            using (var geometryContext = geometry.Open())
            {
                geometryContext.BeginFigure(PointAt(center, radius, start), isFilled: false, isClosed: false);
                geometryContext.ArcTo(
                    PointAt(center, radius, end), new Size(radius, radius), 0, isLargeArc: false,
                    SweepDirection.Clockwise, isStroked: true, isSmoothJoin: false);
            }
            geometry.Freeze();
            context.DrawGeometry(null, pen, geometry);
        }
    }

    /// <summary>A point on the circle at an angle measured clockwise from 3 o'clock (screen y points down).</summary>
    private static Point PointAt(Point center, double radius, double angle) =>
        new(center.X + radius * Math.Cos(angle), center.Y + radius * Math.Sin(angle));
}
