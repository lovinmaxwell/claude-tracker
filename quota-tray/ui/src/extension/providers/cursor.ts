import type { ProviderSnapshot, UsageWindow } from "../../lib/types";

export const CURSOR_USAGE_PATH =
  "/aiserver.v1.DashboardService/GetCurrentPeriodUsage";
export const CURSOR_API_BASE = "https://api2.cursor.sh";

export function cursorHeadlinePercent(planUsage: Record<string, unknown>): number | null {
  const total = asFiniteNumber(planUsage.totalPercentUsed);
  if (total != null) {
    return total;
  }
  const auto = asFiniteNumber(planUsage.autoPercentUsed);
  const api = asFiniteNumber(planUsage.apiPercentUsed);
  if (auto != null && api != null) {
    return Math.max(auto, api);
  }
  if (auto != null) {
    return auto;
  }
  if (api != null) {
    return api;
  }
  const spend = asFiniteNumber(planUsage.totalSpend);
  const limit = asFiniteNumber(planUsage.limit);
  if (spend != null && limit != null && limit > 0) {
    return (100.0 * spend) / limit;
  }
  return null;
}

export function parseCursorUsageBody(body: unknown): ProviderSnapshot {
  const root = asObject(body);
  const plan = asObject(root.planUsage) ?? root;
  const headline = cursorHeadlinePercent(plan);
  const windows: UsageWindow[] = [];
  const auto = asFiniteNumber(plan.autoPercentUsed);
  if (auto != null) {
    windows.push({
      id: "auto",
      label: "Auto / Composer",
      kind: { Percent: { used: auto } },
      resets_at: null,
    });
  }
  const api = asFiniteNumber(plan.apiPercentUsed);
  if (api != null) {
    windows.push({
      id: "api",
      label: "API",
      kind: { Percent: { used: api } },
      resets_at: null,
    });
  }
  if (windows.length === 0) {
    windows.push({
      id: "period",
      label: "Current period",
      kind: { Percent: { used: headline } },
      resets_at: null,
    });
  }
  if (
    headline == null &&
    plan.totalPercentUsed == null &&
    plan.autoPercentUsed == null &&
    plan.apiPercentUsed == null &&
    plan.totalSpend == null
  ) {
    throw new Error("GetCurrentPeriodUsage: no planUsage percent/spend fields");
  }
  return {
    provider: "Cursor",
    fetched_at: new Date().toISOString(),
    windows,
    headline_percent: headline,
    stale: false,
    error: null,
  };
}

export async function fetchCursorUsage(
  token: string,
  baseUrl = CURSOR_API_BASE,
  fetcher: typeof fetch = fetch
): Promise<ProviderSnapshot> {
  const url = `${baseUrl.replace(/\/$/, "")}${CURSOR_USAGE_PATH}`;
  const res = await fetcher(url, {
    method: "POST",
    headers: {
      Authorization: `Bearer ${token}`,
      "Content-Type": "application/json",
    },
    body: "{}",
  });
  const text = await res.text();
  if (!res.ok) {
    throw new Error(`Cursor HTTP ${res.status}`);
  }
  return parseCursorUsageBody(JSON.parse(text) as unknown);
}

function asObject(v: unknown): Record<string, unknown> {
  if (v && typeof v === "object" && !Array.isArray(v)) {
    return v as Record<string, unknown>;
  }
  return {};
}

function asFiniteNumber(v: unknown): number | null {
  return typeof v === "number" && Number.isFinite(v) ? v : null;
}
