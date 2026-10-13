using System;
using System.Drawing;
using System.Drawing.Drawing2D;
using System.Drawing.Imaging;
using System.Runtime.InteropServices;
using ClaudeUsageCore;

namespace ClaudeWidget;

/// <summary>
/// Draws the notification-area icon in the Claude Tracker style: a ring of 12 pills around the creature.
/// The pills fill clockwise as usage grows (the last quarter turns coral) and the creature fills from its feet up.
/// </summary>
public static class TrayIconRenderer
{
    private const int Size = 32;
    private const int PillCount = 12;

    /// <summary>Renders the icon for a peak usage (null when unknown). The caller owns the returned icon.</summary>
    public static Icon Render(double? used, bool isStale)
    {
        using var bitmap = new Bitmap(Size, Size, PixelFormat.Format32bppArgb);
        using (var graphics = Graphics.FromImage(bitmap))
        {
            graphics.SmoothingMode = SmoothingMode.AntiAlias;
            graphics.PixelOffsetMode = PixelOffsetMode.Half;
            graphics.Clear(Color.Transparent);
            DrawMascot(graphics, used);
            DrawPills(graphics, used, isStale);
        }

        // GetHicon hands over a native handle; clone the icon so we can free the handle immediately.
        var handle = bitmap.GetHicon();
        try
        {
            using var temporary = Icon.FromHandle(handle);
            return (Icon)temporary.Clone();
        }
        finally
        {
            DestroyIcon(handle);
        }
    }

    /// <summary>The creature, three quarters of the icon wide, filled from the feet up.</summary>
    private static void DrawMascot(Graphics graphics, double? used)
    {
        var cell = Size * 0.75f / 16;
        var spentRows = IconFill.LitSteps(used);
        using var spent = new SolidBrush(ToDrawing(Mascot.Spent));
        using var unspent = new SolidBrush(ToDrawing(Mascot.Unspent));
        using var eye = new SolidBrush(ToDrawing(Mascot.Eye));

        for (var y = 0; y < Mascot.Art.Length; y++)
        {
            for (var x = 0; x < Mascot.Art[y].Length; x++)
            {
                var character = Mascot.Art[y][x];
                if (character == '.')
                {
                    continue;
                }
                var brush = character == 'E' ? eye : 13 - y < spentRows ? spent : unspent;
                graphics.FillRectangle(brush, Size / 2f + (x - 8) * cell, Size / 2f + (y - 8) * cell, cell + 0.3f, cell + 0.3f);
            }
        }
    }

    /// <summary>Twelve short arcs around the edge, matching the original icon's proportions.</summary>
    private static void DrawPills(Graphics graphics, double? used, bool isStale)
    {
        const float radius = 13.4f;
        const float thickness = 2 * 1.35f;
        var filled = IconFill.LitSteps(used);
        var step = 2 * Math.PI / PillCount;
        var bounds = new RectangleF(Size / 2f - radius, Size / 2f - radius, radius * 2, radius * 2);
        var alpha = isStale ? 140 : 255;

        for (var index = 0; index < PillCount; index++)
        {
            var color = index switch
            {
                _ when index >= filled => Color.FromArgb(115, ToDrawing(Mascot.Unspent)),
                >= 9 => Color.FromArgb(alpha, ToDrawing(Mascot.Warning)),
                _ => Color.FromArgb(alpha, ToDrawing(Mascot.Spent)),
            };
            using var pen = new Pen(color, thickness) { StartCap = LineCap.Round, EndCap = LineCap.Round };
            var middle = -Math.PI / 2 + (index + 0.5) * step;
            var startDegrees = (float)((middle - 0.10) * 180 / Math.PI);
            graphics.DrawArc(pen, bounds, startDegrees, (float)(0.20 * 180 / Math.PI));
        }
    }

    /// <summary>The WPF colour as a System.Drawing colour.</summary>
    private static Color ToDrawing(System.Windows.Media.Color color) => Color.FromArgb(color.R, color.G, color.B);

    [DllImport("user32.dll", SetLastError = true)]
    private static extern bool DestroyIcon(IntPtr handle);
}
