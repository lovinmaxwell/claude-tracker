namespace ClaudeUsageCore;

/// <summary>
/// The Claude Tracker icon has 12 pills around its ring and 12 body rows on the creature; both light up
/// together as usage grows. This is the one place that turns a percentage into that count.
/// </summary>
public static class IconFill
{
    public const int Steps = 12;

    /// <summary>How many of the 12 steps are lit; unknown (null) lights none, never a made-up reading.</summary>
    public static int LitSteps(double? percent) =>
        (int)Math.Round(Math.Clamp(percent ?? 0, 0, 100) / 100 * Steps, MidpointRounding.AwayFromZero);
}
