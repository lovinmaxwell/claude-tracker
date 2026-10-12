import type { ProviderSnapshot, TrayState, UsageWindow } from "./types";

/** Resting opacity of the floating widget until the pointer is over it. */
export const WIDGET_REST_OPACITY = 0.2;

/** Rings drawn at once; extra windows are summarised as "+N more". */
export const MAX_WIDGET_RINGS = 3;

const WINDOW_DURATION_MS: Record<string, number> = {
  five_hour: 5 * 60 * 60 * 1000,
  seven_day: 7 * 24 * 60 * 60 * 1000,
};

export type WidgetTone = "ok" | "watch" | "hot" | "stale" | "wait";

export interface WidgetRing {
  id: string;
  label: string;
  /** 0..100, or null when the vendor did not report a percentage. */
  used: number | null;
  resetsAt: Date | null;
  /** Colour token name, resolved against the active palette in CSS. */
  accent: "claude" | "cursor" | "copilot";
  tone: WidgetTone;
}

export interface WidgetStatus {
  tone: WidgetTone;
  headline: string;
  detail: string | null;
}

/**
 * Reads `resets_at` from either wire shape: an RFC 3339 string (Chrome
 * extension) or the `time` crate's default tuple
 * `[year, ordinal, hour, minute, second, nano, offH, offM, offS]` (Tauri IPC).
 */
export function parseResetsAt(raw: unknown): Date | null {
  if (typeof raw === "string") {
    const ms = Date.parse(raw);
    return Number.isNaN(ms) ? null : new Date(ms);
  }
  if (Array.isArray(raw) && raw.length >= 9 && raw.every((n) => typeof n === "number")) {
    const [year, ordinal, hour, minute, second, nano, offH, offM, offS] = raw as number[];
    const utcMs =
      Date.UTC(year, 0, ordinal, hour, minute, second, Math.floor(nano / 1e6)) -
      (offH * 3600 + offM * 60 + offS) * 1000;
    return new Date(utcMs);
  }
  return null;
}

/** "2h 14m", "38m", "3d 4h"; null once the moment has passed. */
export function formatCountdown(target: Date | null, now: Date): string | null {
  if (!target) {
    return null;
  }
  const totalMinutes = Math.floor((target.getTime() - now.getTime()) / 60_000);
  if (totalMinutes < 0) {
    return null;
  }
  if (totalMinutes < 1) {
    return "<1m";
  }
  const days = Math.floor(totalMinutes / 1440);
  const hours = Math.floor((totalMinutes % 1440) / 60);
  const minutes = totalMinutes % 60;
  if (days > 0) {
    return hours > 0 ? `${days}d ${hours}h` : `${days}d`;
  }
  if (hours > 0) {
    return minutes > 0 ? `${hours}h ${minutes}m` : `${hours}h`;
  }
  return `${minutes}m`;
}

/** "Tue 1:00 AM" style absolute reset time in the viewer's locale. */
export function formatResetClock(target: Date | null, now: Date): string | null {
  if (!target) {
    return null;
  }
  const sameDay = target.toDateString() === now.toDateString();
  const time = target.toLocaleTimeString([], { hour: "numeric", minute: "2-digit" });
  return sameDay ? time : `${target.toLocaleDateString([], { weekday: "short" })} ${time}`;
}

/** "just now", "42s ago", "5m ago". */
export function formatAge(fetchedAt: Date | null, now: Date): string | null {
  if (!fetchedAt) {
    return null;
  }
  const seconds = Math.max(0, Math.floor((now.getTime() - fetchedAt.getTime()) / 1000));
  if (seconds < 10) {
    return "just now";
  }
  if (seconds < 60) {
    return `${seconds}s ago`;
  }
  return `${Math.floor(seconds / 60)}m ago`;
}

function toneForUsed(used: number | null): WidgetTone {
  if (used === null) {
    return "wait";
  }
  if (used >= 85) {
    return "hot";
  }
  if (used >= 65) {
    return "watch";
  }
  return "ok";
}

/**
 * Fraction (0..1) of a rolling window that has already elapsed, or null when
 * the window length or reset moment is unknown.
 */
export function elapsedFraction(windowId: string, resetsAt: Date | null, now: Date): number | null {
  const duration = WINDOW_DURATION_MS[windowId];
  if (!duration || !resetsAt) {
    return null;
  }
  const remaining = resetsAt.getTime() - now.getTime();
  if (remaining < 0 || remaining > duration) {
    return null;
  }
  return 1 - remaining / duration;
}

/**
 * Hours until a window runs dry if usage continues at the pace seen so far;
 * null when there is not enough signal (under 10% of the window elapsed) or
 * the reset arrives first.
 */
export function projectedHoursToLimit(
  used: number,
  windowId: string,
  resetsAt: Date | null,
  now: Date
): number | null {
  const elapsed = elapsedFraction(windowId, resetsAt, now);
  const duration = WINDOW_DURATION_MS[windowId];
  if (elapsed === null || !duration || elapsed < 0.1 || used <= 0) {
    return null;
  }
  const elapsedMs = elapsed * duration;
  const percentPerMs = used / elapsedMs;
  const msToLimit = (100 - used) / percentPerMs;
  const msToReset = resetsAt!.getTime() - now.getTime();
  return msToLimit < msToReset ? msToLimit / 3_600_000 : null;
}

/** Ring colours by position, so concentric rings stay distinguishable. */
const RING_ACCENTS: WidgetRing["accent"][] = ["claude", "cursor", "copilot"];

/** Picks the percentage windows worth a ring; unsupported kinds are skipped. */
export function ringsFor(snap: ProviderSnapshot): WidgetRing[] {
  const rings: WidgetRing[] = snap.windows
    .filter((w: UsageWindow) => "Percent" in w.kind)
    .map((w, i) => {
      const used = (w.kind as { Percent: { used: number | null } }).Percent.used;
      return {
        id: w.id,
        label: w.label,
        used: used === null || Number.isNaN(used) ? null : Math.min(100, Math.max(0, used)),
        resetsAt: parseResetsAt(w.resets_at),
        accent: RING_ACCENTS[i % RING_ACCENTS.length],
        tone: toneForUsed(used),
      };
    });
  if (rings.length === 0 && snap.headline_percent !== null) {
    rings.push({
      id: "headline",
      label: "Usage",
      used: snap.headline_percent,
      resetsAt: null,
      accent: RING_ACCENTS[0],
      tone: toneForUsed(snap.headline_percent),
    });
  }
  return rings;
}

/** The provider the widget shows by default: Claude first, else the first one with data. */
export function defaultProvider(state: TrayState | null): ProviderSnapshot | null {
  const providers = state?.providers ?? [];
  return (
    providers.find((p) => p.provider === "Claude" && ringsFor(p).length > 0) ??
    providers.find((p) => ringsFor(p).length > 0) ??
    providers[0] ??
    null
  );
}

/** The ring with the highest usage; what the resting pill leads with. */
export function peakRing(rings: WidgetRing[]): WidgetRing | null {
  return rings.reduce<WidgetRing | null>(
    (best, r) => (r.used !== null && (best === null || r.used > (best.used ?? -1)) ? r : best),
    null
  );
}

/** One plain-English sentence describing where the user stands. */
export function summarize(snap: ProviderSnapshot | null, rings: WidgetRing[], now: Date): WidgetStatus {
  if (!snap || rings.every((r) => r.used === null)) {
    return { tone: "wait", headline: "Waiting for a reading", detail: null };
  }
  if (snap.error) {
    return { tone: "stale", headline: "Can't refresh", detail: "Showing the last reading" };
  }
  if (snap.stale) {
    return { tone: "stale", headline: "Out of date", detail: "Showing the last reading" };
  }

  const peak = peakRing(rings)!;
  const used = peak.used!;
  const reset = formatCountdown(peak.resetsAt, now);

  if (used >= 100) {
    return {
      tone: "hot",
      headline: "Limit reached",
      detail: reset ? `${peak.label} resets in ${reset}` : null,
    };
  }

  const hours = projectedHoursToLimit(used, peak.id, peak.resetsAt, now);
  if (hours !== null) {
    const h = Math.floor(hours);
    const m = Math.round((hours - h) * 60);
    const eta = h > 0 ? `${h}h ${m}m` : `${Math.max(1, m)}m`;
    return {
      tone: used >= 65 ? "hot" : "watch",
      headline: "Ahead of pace",
      detail: `${peak.label} runs out in ~${eta} at this rate`,
    };
  }

  if (used >= 85) {
    return {
      tone: "hot",
      headline: "Nearly out",
      detail: reset ? `${peak.label} resets in ${reset}` : null,
    };
  }
  return {
    tone: "ok",
    headline: "On track",
    detail: reset ? `${peak.label} resets in ${reset}` : null,
  };
}
