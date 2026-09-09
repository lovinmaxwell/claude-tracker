import type { ClaudeHeadline } from "../../lib/config";
import type { ProviderSnapshot, UsageWindow } from "../../lib/types";

export const CLAUDE_USAGE_PATH = "/api/oauth/usage";
export const CLAUDE_API_BASE = "https://api.anthropic.com";
export const ANTHROPIC_BETA_OAUTH = "oauth-2025-04-20";

export function parseClaudeUsageBody(
  body: unknown,
  headline: ClaudeHeadline
): ProviderSnapshot {
  const root = (body ?? {}) as {
    five_hour?: { utilization?: number; resets_at?: string };
    seven_day?: { utilization?: number; resets_at?: string };
  };
  const windows: UsageWindow[] = [];
  if (root.five_hour) {
    windows.push(mapWindow("five_hour", "5-hour", root.five_hour));
  }
  if (root.seven_day) {
    windows.push(mapWindow("seven_day", "7-day", root.seven_day));
  }
  return {
    provider: "Claude",
    fetched_at: new Date().toISOString(),
    windows,
    headline_percent: headlineFromWindows(windows, headline),
    stale: false,
    error: null,
  };
}

export async function fetchClaudeUsage(
  token: string,
  headline: ClaudeHeadline,
  baseUrl = CLAUDE_API_BASE,
  fetcher: typeof fetch = fetch
): Promise<ProviderSnapshot> {
  const url = `${baseUrl.replace(/\/$/, "")}${CLAUDE_USAGE_PATH}`;
  const res = await fetcher(url, {
    headers: {
      Authorization: `Bearer ${token}`,
      "anthropic-beta": ANTHROPIC_BETA_OAUTH,
    },
  });
  const text = await res.text();
  if (!res.ok) {
    throw new Error(`Claude HTTP ${res.status}`);
  }
  return parseClaudeUsageBody(JSON.parse(text) as unknown, headline);
}

function mapWindow(
  id: string,
  label: string,
  dto: { utilization?: number; resets_at?: string }
): UsageWindow {
  const used =
    typeof dto.utilization === "number" && Number.isFinite(dto.utilization)
      ? Math.min(100, Math.max(0, dto.utilization))
      : null;
  return {
    id,
    label,
    kind: { Percent: { used } },
    resets_at: dto.resets_at ?? null,
  };
}

function percentUsed(window: UsageWindow): number | null {
  return "Percent" in window.kind ? window.kind.Percent.used : null;
}

export function headlineFromWindows(
  windows: UsageWindow[],
  metric: ClaudeHeadline
): number | null {
  const five = windows.find((w) => w.id === "five_hour");
  const seven = windows.find((w) => w.id === "seven_day");
  const fivePct = five ? percentUsed(five) : null;
  const sevenPct = seven ? percentUsed(seven) : null;
  if (metric === "FiveHour") {
    return fivePct;
  }
  if (metric === "SevenDay") {
    return sevenPct;
  }
  if (fivePct != null && sevenPct != null) {
    return Math.max(fivePct, sevenPct);
  }
  return fivePct ?? sevenPct;
}
