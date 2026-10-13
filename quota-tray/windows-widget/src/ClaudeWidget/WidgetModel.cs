using System;
using System.Collections.Generic;
using System.Linq;
using ClaudeUsageCore;

namespace ClaudeWidget;

/// <summary>One ring: a usage window plus the colour it is drawn in.</summary>
public sealed record RingModel(string Id, string Label, double? Used, DateTimeOffset? ResetsAt, RingColor Color)
{
    /// <summary>0..1 fill; unknown usage draws nothing.</summary>
    public double Fraction => (Used ?? 0) / 100;
}

/// <summary>Everything the views need, derived once per tick from the store's state.</summary>
public sealed record WidgetModel(IReadOnlyList<RingModel> Rings, RingModel? Peak, UsageStatus Status, string? Age)
{
    /// <summary>Builds the model from the latest reading (or its absence) and any error.</summary>
    public static WidgetModel From(UsageSnapshot? snapshot, UsageException? error, DateTimeOffset now)
    {
        var rings = (snapshot?.Windows ?? [])
            .Select((window, index) => new RingModel(
                window.Id,
                window.Label,
                window.Used,
                window.ResetsAt,
                UsageAnalysis.ToneFor(window.Used) == Tone.Hot ? RingColor.Hot : index == 0 ? RingColor.Session : RingColor.Week))
            .ToList();
        var peakWindow = snapshot is null ? null : UsageAnalysis.Peak(snapshot.Windows);
        return new WidgetModel(
            rings,
            rings.FirstOrDefault(r => r.Id == peakWindow?.Id),
            UsageAnalysis.Summarize(snapshot, snapshot is not null && error is not null, error, now),
            Formatters.Age(snapshot?.FetchedAt, now));
    }
}
