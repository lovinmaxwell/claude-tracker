using System;
using System.Windows.Media;
using ClaudeUsageCore;
using Microsoft.Win32;

namespace ClaudeWidget;

/// <summary>Which colour a ring is drawn in.</summary>
public enum RingColor
{
    Session,
    Week,
    Hot,
}

/// <summary>Colours that follow the Windows light/dark setting.</summary>
public sealed record Theme(Color Card, Color Border, Color Ink, Color Muted)
{
    private static readonly Theme Light = new(
        Color.FromArgb(0xF0, 0xFB, 0xF7, 0xF2), Color.FromArgb(0x30, 0, 0, 0),
        Color.FromRgb(0x2C, 0x2C, 0x2C), Color.FromRgb(0x6B, 0x5D, 0x50));

    private static readonly Theme Dark = new(
        Color.FromArgb(0xF0, 0x26, 0x1E, 0x19), Color.FromArgb(0x30, 0xFF, 0xFF, 0xFF),
        Color.FromRgb(0xF5, 0xED, 0xE6), Color.FromRgb(0xB8, 0xA9, 0x9A));

    /// <summary>The theme currently in force; call <see cref="Refresh"/> to pick up a Windows change.</summary>
    public static Theme Current { get; private set; } = Detect();

    /// <summary>Re-reads the Windows app theme.</summary>
    public static void Refresh() => Current = Detect();

    /// <summary>Reads HKCU AppsUseLightTheme; light when the value is missing.</summary>
    private static Theme Detect()
    {
        using var key = Registry.CurrentUser.OpenSubKey(@"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize");
        return key?.GetValue("AppsUseLightTheme") is 0 ? Dark : Light;
    }

    /// <summary>The ring colours: terracotta for the session, sage for the week, red when nearly out.</summary>
    public static Color Resolve(RingColor color) => color switch
    {
        RingColor.Session => Color.FromRgb(0xC9, 0x64, 0x42),
        RingColor.Week => Color.FromRgb(0x5A, 0x7A, 0x6A),
        _ => Color.FromRgb(0xD9, 0x4B, 0x34),
    };

    /// <summary>The colour of a status chip for a given tone.</summary>
    public Color ForTone(Tone tone) => tone switch
    {
        Tone.Ok => Resolve(RingColor.Week),
        Tone.Watch => Color.FromRgb(0xD9, 0xA1, 0x3B),
        Tone.Hot => Resolve(RingColor.Hot),
        _ => Muted,
    };

    /// <summary>A frozen brush, which WPF can share across threads and draw cheaply.</summary>
    public static SolidColorBrush Brush(Color color)
    {
        var brush = new SolidColorBrush(color);
        brush.Freeze();
        return brush;
    }
}
