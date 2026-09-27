import type { ClaudeHeadline } from "../../lib/config";
import type { ProviderSnapshot, UsageWindow } from "../../lib/types";

export const CLAUDE_USAGE_PATH = "/api/oauth/usage";
export const CLAUDE_API_BASE = "https://api.anthropic.com";
export const ANTHROPIC_BETA_OAUTH = "oauth-2025-04-20";
export const CLAUDE_WEB_BASE = "https://claude.ai";
export const CLAUDE_ORGS_PATH = "/api/organizations";

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

/** Picks the org uuid a signed-in claude.ai session's usage lives under. */
export function pickClaudeOrgUuid(body: unknown): string | null {
  if (!Array.isArray(body)) {
    return null;
  }
  for (const entry of body) {
    if (entry && typeof entry === "object" && typeof (entry as { uuid?: unknown }).uuid === "string") {
      return (entry as { uuid: string }).uuid;
    }
  }
  return null;
}

export async function fetchClaudeOrgUuid(
  baseUrl = CLAUDE_WEB_BASE,
  fetcher: typeof fetch = fetch
): Promise<string | null> {
  const url = `${baseUrl.replace(/\/$/, "")}${CLAUDE_ORGS_PATH}`;
  const res = await fetcher(url, { credentials: "include" });
  if (!res.ok) {
    return null;
  }
  const body = (await res.json()) as unknown;
  return pickClaudeOrgUuid(body);
}

/** Mirrors the OAuth usage shape (five_hour/seven_day) that claude.ai's own web app reads via its session cookie. */
export async function fetchClaudeUsageBySession(
  headline: ClaudeHeadline,
  baseUrl = CLAUDE_WEB_BASE,
  fetcher: typeof fetch = fetch
): Promise<ProviderSnapshot> {
  const orgUuid = await fetchClaudeOrgUuid(baseUrl, fetcher);
  if (!orgUuid) {
    throw new Error("Claude: no organization found for this session");
  }
  const url = `${baseUrl.replace(/\/$/, "")}${CLAUDE_ORGS_PATH}/${orgUuid}/usage`;
  const res = await fetcher(url, { credentials: "include" });
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
