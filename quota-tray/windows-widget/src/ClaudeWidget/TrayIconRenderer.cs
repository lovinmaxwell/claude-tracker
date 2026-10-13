using System;
using System.Drawing;
using System.Drawing.Drawing2D;
using System.Drawing.Imaging;
using System.Runtime.InteropServices;

namespace ClaudeWidget;

/// <summary>Draws the notification-area icon: a small ring filled to the peak usage.</summary>
public static class TrayIconRenderer
{
    private const int Size = 32;

    /// <summary>Renders a ring icon for the given ring (an empty track when unknown). The caller owns the icon.</summary>
    public static Icon Render(RingModel? peak)
    {
        using var bitmap = new Bitmap(Size, Size, PixelFormat.Format32bppArgb);
        using (var graphics = Graphics.FromImage(bitmap))
        {
            graphics.SmoothingMode = SmoothingMode.AntiAlias;
            graphics.Clear(Color.Transparent);

            using var track = new Pen(Color.FromArgb(150, 140, 140, 140), 5);
            graphics.DrawEllipse(track, 4, 4, 24, 24);

            if (peak?.Used is { } used and > 0)
            {
                var rgb = Theme.Resolve(peak.Color);
                using var arc = new Pen(Color.FromArgb(rgb.R, rgb.G, rgb.B), 5)
                {
                    StartCap = LineCap.Round,
                    EndCap = LineCap.Round,
                };
                if (used >= 99.9)
                {
                    graphics.DrawEllipse(arc, 4, 4, 24, 24);
                }
                else
                {
                    graphics.DrawArc(arc, 4, 4, 24, 24, -90f, (float)(used / 100 * 360));
                }
            }
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

    [DllImport("user32.dll", SetLastError = true)]
    private static extern bool DestroyIcon(IntPtr handle);
}
