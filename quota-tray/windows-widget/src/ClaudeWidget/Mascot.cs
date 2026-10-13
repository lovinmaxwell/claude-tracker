using System;
using System.Windows;
using System.Windows.Media;
using ClaudeUsageCore;

namespace ClaudeWidget;

/// <summary>
/// The pixel creature from the Claude Tracker icon, drawn from the same 16x16 grid as the original
/// (PixelMascot in the original Claude Tracker by Bhargav Detroja; see the root README credits). Like the
/// original, it fills with terracotta from the feet up.
/// </summary>
public static class Mascot
{
    /// <summary>B is body, E is an eye, a dot is empty.</summary>
    public static readonly string[] Art =
    [
        "................",
        "................",
        "...BBBBBBBBBB...",
        "...BBBBBBBBBB...",
        "...BEEBBBBEEB...",
        "...BEEBBBBEEB...",
        ".BBBBBBBBBBBBBB.",
        ".BBBBBBBBBBBBBB.",
        ".BBBBBBBBBBBBBB.",
        "...BBBBBBBBBB...",
        "...BBBBBBBBBB...",
        "...BB.BB.BB.BB..",
        "...BB.BB.BB.BB..",
        "...BB.BB.BB.BB..",
        "................",
        "................",
    ];

    /// <summary>Terracotta, the spent part of the body.</summary>
    public static readonly Color Spent = Color.FromRgb(0xC9, 0x64, 0x42);

    /// <summary>The unspent part of the body, and the empty pills of the ring.</summary>
    public static readonly Color Unspent = Color.FromRgb(0xDF, 0xD2, 0xC4);

    public static readonly Color Eye = Color.FromRgb(0x2C, 0x2C, 0x2C);

    /// <summary>The coral the last quarter of the pill ring turns as a warning.</summary>
    public static readonly Color Warning = Color.FromRgb(0xD9, 0x4B, 0x34);

    /// <summary>The creature occupies grid rows 2 through 13, which is what the fill spans.</summary>
    private const int LastBodyRow = 13;

    private static readonly Brush SpentBrush = Theme.Brush(Spent);
    private static readonly Brush UnspentBrush = Theme.Brush(Unspent);
    private static readonly Brush EyeBrush = Theme.Brush(Eye);

    /// <summary>
    /// Draws the creature centred on <paramref name="center"/>. <paramref name="fillPercent"/> is percent spent,
    /// filled from the feet up; null means unknown and shows the unspent body, never a made-up reading.
    /// </summary>
    public static void Draw(DrawingContext context, Point center, double cell, double? fillPercent)
    {
        var spentRows = IconFill.LitSteps(fillPercent);
        for (var y = 0; y < Art.Length; y++)
        {
            for (var x = 0; x < Art[y].Length; x++)
            {
                var character = Art[y][x];
                if (character == '.')
                {
                    continue;
                }
                var brush = character == 'E' ? EyeBrush : LastBodyRow - y < spentRows ? SpentBrush : UnspentBrush;
                // The art is centred on cell (8, 8); the small overlap hides hairline seams between cells.
                var rect = new Rect(center.X + (x - 8) * cell, center.Y + (y - 8) * cell, cell + 0.4, cell + 0.4);
                context.DrawRectangle(brush, null, rect);
            }
        }
    }
}
