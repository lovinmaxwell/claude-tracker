import type { TrayState } from "./types";

/** Believable readings for previewing the widget in a plain browser. */
export function mockTrayState(now: Date): TrayState {
  const inHours = (h: number) => new Date(now.getTime() + h * 3_600_000).toISOString();
  const fetched = new Date(now.getTime() - 20_000).toISOString();
  return {
    shared_mascot_fill: 62,
    providers: [
      {
        provider: "Claude",
        fetched_at: fetched,
        stale: false,
        error: null,
        headline_percent: 62,
        windows: [
          { id: "five_hour", label: "5-hour", kind: { Percent: { used: 38 } }, resets_at: inHours(2.4) },
          { id: "seven_day", label: "7-day", kind: { Percent: { used: 62 } }, resets_at: inHours(24 * 3.2) },
        ],
      },
      {
        provider: "Cursor",
        fetched_at: fetched,
        stale: false,
        error: null,
        headline_percent: 27,
        windows: [{ id: "plan", label: "Plan", kind: { Percent: { used: 27 } }, resets_at: inHours(24 * 11) }],
      },
      {
        provider: "Copilot",
        fetched_at: fetched,
        stale: false,
        error: null,
        headline_percent: 91,
        windows: [{ id: "premium", label: "Premium", kind: { Percent: { used: 91 } }, resets_at: inHours(24 * 6) }],
      },
    ],
  };
}
