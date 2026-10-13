using System;
using System.Collections.Generic;
using System.Linq;
using System.Windows;
using System.Windows.Media;
using System.Windows.Media.Effects;

namespace ClaudeWidget;

/// <summary>Concentric progress rings, outermost first, with the Claude Tracker creature in the middle.</summary>
public sealed class RingsControl : FrameworkElement
{
    private static readonly Brush HitBackdrop = Theme.Brush(Color.FromArgb(1, 0, 0, 0));

    private IReadOnlyList<RingModel> _rings = [];
    private double _size = 96;
    private double _stroke = 9;
    private double _gap = 3;
    private double _trackAlpha = 0.1;

    /// <summary>Sets the geometry; the card uses faint tracks, the resting rings need bolder ones.</summary>
    public void Configure(double size, double stroke, double gap, double trackAlpha, bool shadow)
    {
        (_size, _stroke, _gap, _trackAlpha) = (size, stroke, gap, trackAlpha);
        Effect = shadow
            ? new DropShadowEffect { BlurRadius = 4, ShadowDepth = 1, Opacity = 0.4, Color = Colors.Black }
            : null;
        InvalidateMeasure();
        InvalidateVisual();
    }

    /// <summary>Replaces the rings and redraws.</summary>
    public void SetRings(IReadOnlyList<RingModel> rings)
    {
        _rings = rings;
        InvalidateVisual();
    }

    protected override Size MeasureOverride(Size availableSize) => new(_size, _size);

    protected override void OnRender(DrawingContext drawingContext)
    {
        var theme = Theme.Current;
        var center = new Point(_size / 2, _size / 2);

        // Nearly invisible, but it makes the whole disc hit-testable so hovering the gaps still counts.
        drawingContext.DrawEllipse(HitBackdrop, null, center, _size / 2, _size / 2);

        var trackBrush = Theme.Brush(Color.FromArgb((byte)(_trackAlpha * 255), theme.Ink.R, theme.Ink.G, theme.Ink.B));
        var trackPen = new Pen(trackBrush, _stroke);
        trackPen.Freeze();

        var drawn = Math.Max(_rings.Count, 1);
        for (var index = 0; index < drawn; index++)
        {
            var radius = (_size - _stroke) / 2 - index * (_stroke + _gap);
            if (radius <= _stroke / 2)
            {
                break;
            }
            drawingContext.DrawEllipse(null, trackPen, center, radius, radius);

            if (index < _rings.Count && _rings[index] is { Used: not null } ring)
            {
                DrawProgress(drawingContext, center, radius, ring);
            }
        }

        DrawMascot(drawingContext, center, drawn);
    }

    /// <summary>Draws the filled arc, starting at 12 o'clock and running clockwise.</summary>
    private void DrawProgress(DrawingContext context, Point center, double radius, RingModel ring)
    {
        // A sliver stays visible at tiny values so "1%" never looks like "unknown".
        var fraction = Math.Max(ring.Fraction, ring.Fraction > 0 ? 0.015 : 0);
        if (fraction <= 0)
        {
            return;
        }
        var pen = new Pen(Theme.Brush(Theme.Resolve(ring.Color)), _stroke)
        {
            StartLineCap = PenLineCap.Round,
            EndLineCap = PenLineCap.Round,
        };
        pen.Freeze();

        if (fraction >= 0.999)
        {
            context.DrawEllipse(null, pen, center, radius, radius);
            return;
        }

        var sweep = fraction * 2 * Math.PI;
        var start = new Point(center.X, center.Y - radius);
        var end = new Point(
            center.X + radius * Math.Sin(sweep),
            center.Y - radius * Math.Cos(sweep));

        var geometry = new StreamGeometry();
        using (var geometryContext = geometry.Open())
        {
            geometryContext.BeginFigure(start, isFilled: false, isClosed: false);
            geometryContext.ArcTo(end, new Size(radius, radius), 0, fraction > 0.5, SweepDirection.Clockwise, isStroked: true, isSmoothJoin: false);
        }
        geometry.Freeze();
        context.DrawGeometry(null, pen, geometry);
    }

    /// <summary>Draws the creature, sized to sit inside the innermost ring with a little breathing room.</summary>
    private void DrawMascot(DrawingContext context, Point center, int ringCount)
    {
        var innerRadius = _size / 2 - _stroke - (ringCount - 1) * (_stroke + _gap);
        var cell = Math.Max(1, innerRadius * 2 * 0.7 / 14);
        double? peak = _rings.Max(r => r.Used);
        Mascot.Draw(context, center, cell, peak);
    }
}
