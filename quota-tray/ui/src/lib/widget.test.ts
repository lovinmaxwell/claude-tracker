import { describe, expect, it } from "vitest";
import {
  defaultProvider,
  elapsedFraction,
  formatAge,
  formatCountdown,
  parseResetsAt,
  peakRing,
  projectedHoursToLimit,
  ringsFor,
  summarize,
} from "./widget";
import type { ProviderSnapshot, TrayState } from "./types";

const NOW = new Date("2026-10-12T10:00:00Z");
const hoursFromNow = (h: number) => new Date(NOW.getTime() + h * 3_600_000);

function claude(overrides: Partial<ProviderSnapshot> = {}, fiveUsed = 12, sevenUsed = 40): ProviderSnapshot {
  return {
    provider: "Claude",
    fetched_at: NOW.toISOString(),
    stale: false,
    error: null,
    headline_percent: Math.max(fiveUsed, sevenUsed),
    windows: [
      {
        id: "five_hour",
        label: "5-hour",
        kind: { Percent: { used: fiveUsed } },
        resets_at: hoursFromNow(4).toISOString(),
      },
      {
        id: "seven_day",
        label: "7-day",
        kind: { Percent: { used: sevenUsed } },
        resets_at: hoursFromNow(24 * 3).toISOString(),
      },
    ],
    ...overrides,
  };
}

describe("parseResetsAt", () => {
  it("reads RFC 3339 strings", () => {
    expect(parseResetsAt("2026-10-12T15:00:00Z")?.toISOString()).toBe("2026-10-12T15:00:00.000Z");
  });

  it("reads the time crate tuple, applying the UTC offset", () => {
    // 2026-10-12 is ordinal day 285. 17:30 at +05:30 is 12:00 UTC.
    expect(parseResetsAt([2026, 285, 17, 30, 0, 0, 5, 30, 0])?.toISOString()).toBe(
      "2026-10-12T12:00:00.000Z"
    );
  });

  it("returns null for junk and null", () => {
    expect(parseResetsAt(null)).toBeNull();
    expect(parseResetsAt("not a date")).toBeNull();
    expect(parseResetsAt([1, 2, 3])).toBeNull();
  });
});

describe("formatCountdown", () => {
  it("formats minutes, hours and days", () => {
    expect(formatCountdown(new Date(NOW.getTime() + 38 * 60_000), NOW)).toBe("38m");
    expect(formatCountdown(new Date(NOW.getTime() + (2 * 60 + 14) * 60_000), NOW)).toBe("2h 14m");
    expect(formatCountdown(hoursFromNow(3 * 24 + 4), NOW)).toBe("3d 4h");
    expect(formatCountdown(hoursFromNow(48), NOW)).toBe("2d");
  });

  it("is null once the moment has passed or when unknown", () => {
    expect(formatCountdown(hoursFromNow(-1), NOW)).toBeNull();
    expect(formatCountdown(null, NOW)).toBeNull();
  });
});

describe("formatAge", () => {
  it("buckets into just now / seconds / minutes", () => {
    expect(formatAge(new Date(NOW.getTime() - 3_000), NOW)).toBe("just now");
    expect(formatAge(new Date(NOW.getTime() - 42_000), NOW)).toBe("42s ago");
    expect(formatAge(new Date(NOW.getTime() - 5 * 60_000), NOW)).toBe("5m ago");
    expect(formatAge(null, NOW)).toBeNull();
  });
});

describe("pace maths", () => {
  it("measures how much of the 5-hour window has elapsed", () => {
    expect(elapsedFraction("five_hour", hoursFromNow(1), NOW)).toBeCloseTo(0.8);
    expect(elapsedFraction("mystery", hoursFromNow(1), NOW)).toBeNull();
    expect(elapsedFraction("five_hour", null, NOW)).toBeNull();
  });

  it("projects the time to the limit only when it lands before the reset", () => {
    // 60% used with 1h left of 5h (4h elapsed) -> 15%/h -> 40% left takes ~2.67h > 1h: reset wins.
    expect(projectedHoursToLimit(60, "five_hour", hoursFromNow(1), NOW)).toBeNull();
    // 80% used after 2h elapsed (3h left) -> 40%/h -> 20% left takes 0.5h.
    expect(projectedHoursToLimit(80, "five_hour", hoursFromNow(3), NOW)).toBeCloseTo(0.5);
  });

  it("refuses to extrapolate from under 10% of the window", () => {
    expect(projectedHoursToLimit(30, "five_hour", hoursFromNow(4.8), NOW)).toBeNull();
  });
});

describe("ringsFor / peakRing / defaultProvider", () => {
  it("builds one ring per percent window and skips other kinds", () => {
    const snap = claude();
    snap.windows.push({
      id: "spend",
      label: "Spend",
      kind: { Currency: { used: 3, limit: 10, code: "USD" } },
      resets_at: null,
    });
    const rings = ringsFor(snap);
    expect(rings.map((r) => r.id)).toEqual(["five_hour", "seven_day"]);
    expect(rings[0].accent).toBe("claude");
  });

  it("falls back to the headline percent when there are no windows", () => {
    const rings = ringsFor(claude({ windows: [], headline_percent: 55 }));
    expect(rings).toHaveLength(1);
    expect(rings[0].used).toBe(55);
  });

  it("never turns an unknown reading into zero", () => {
    const snap = claude();
    snap.windows[0].kind = { Percent: { used: null } };
    const rings = ringsFor(snap);
    expect(rings[0].used).toBeNull();
    expect(rings[0].tone).toBe("wait");
    expect(peakRing(rings)?.id).toBe("seven_day");
  });

  it("prefers Claude, then any provider with data", () => {
    const cursor: ProviderSnapshot = { ...claude(), provider: "Cursor" };
    const state: TrayState = { providers: [cursor, claude()], shared_mascot_fill: null };
    expect(defaultProvider(state)?.provider).toBe("Claude");
    expect(defaultProvider({ providers: [cursor], shared_mascot_fill: null })?.provider).toBe("Cursor");
    expect(defaultProvider(null)).toBeNull();
  });
});

describe("summarize", () => {
  it("says on track when usage is comfortably behind pace", () => {
    const snap = claude({}, 12, 40);
    const status = summarize(snap, ringsFor(snap), NOW);
    expect(status.tone).toBe("ok");
    expect(status.headline).toBe("On track");
    expect(status.detail).toContain("resets in");
  });

  it("warns with an ETA when the current pace would hit the limit first", () => {
    const snap = claude({}, 80, 10);
    snap.windows[0].resets_at = hoursFromNow(3).toISOString();
    const status = summarize(snap, ringsFor(snap), NOW);
    expect(status.headline).toBe("Ahead of pace");
    expect(status.detail).toContain("runs out in ~30m");
  });

  it("reports a reached limit", () => {
    const snap = claude({}, 100, 10);
    expect(summarize(snap, ringsFor(snap), NOW).headline).toBe("Limit reached");
  });

  it("is honest about stale, errored and missing data", () => {
    const stale = claude({ stale: true });
    expect(summarize(stale, ringsFor(stale), NOW).tone).toBe("stale");
    const errored = claude({ error: "401" });
    expect(summarize(errored, ringsFor(errored), NOW).headline).toBe("Can't refresh");
    expect(summarize(null, [], NOW).tone).toBe("wait");
  });
});
