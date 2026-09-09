import type { ProviderSnapshot, UsageWindow } from "../../lib/types";

export const COPILOT_API_BASE = "https://api.github.com";
export const COPILOT_USER_PATH = "/copilot_internal/user";

function usedFromBucket(bucket: Record<string, unknown>): number | null {
  if (bucket.unlimited === true) {
    return null;
  }
  const remaining = bucket.percent_remaining;
  if (typeof remaining !== "number" || !Number.isFinite(remaining)) {
    return null;
  }
  return 100.0 - remaining;
}

export function copilotHeadlinePercent(
  quotaSnapshots: Record<string, unknown>
): number | null {
  const premium = asObject(quotaSnapshots.premium_interactions);
  if (premium) {
    const used = usedFromBucket(premium);
    if (used != null) {
      return used;
    }
  }
  const chat = asObject(quotaSnapshots.chat);
  const completions = asObject(quotaSnapshots.completions);
  const chatUsed = chat ? usedFromBucket(chat) : null;
  const completionsUsed = completions ? usedFromBucket(completions) : null;
  if (chatUsed != null && completionsUsed != null) {
    return Math.max(chatUsed, completionsUsed);
  }
  return chatUsed ?? completionsUsed;
}

export function parseCopilotUser(body: unknown): ProviderSnapshot {
  const root = asObject(body) ?? {};
  const snapshots = asObject(root.quota_snapshots);
  if (!snapshots) {
    throw new Error("missing quota_snapshots");
  }
  const windows: UsageWindow[] = [];
  for (const [id, label] of [
    ["premium_interactions", "Premium"],
    ["chat", "Chat"],
    ["completions", "Completions"],
  ] as const) {
    const bucket = asObject(snapshots[id]);
    if (!bucket) {
      continue;
    }
    windows.push({
      id,
      label,
      kind: { Percent: { used: usedFromBucket(bucket) } },
      resets_at: null,
    });
  }
  if (windows.length === 0) {
    throw new Error("copilot_internal/user: empty quota_snapshots");
  }
  return {
    provider: "Copilot",
    fetched_at: new Date().toISOString(),
    windows,
    headline_percent: copilotHeadlinePercent(snapshots),
    stale: false,
    error: null,
  };
}

export async function fetchCopilotUsage(
  token: string,
  baseUrl = COPILOT_API_BASE,
  fetcher: typeof fetch = fetch
): Promise<ProviderSnapshot> {
  const url = `${baseUrl.replace(/\/$/, "")}${COPILOT_USER_PATH}`;
  const res = await fetcher(url, {
    headers: {
      Authorization: `Bearer ${token}`,
      Accept: "application/json",
      "X-GitHub-Api-Version": "2025-04-01",
      "User-Agent": "QuotaTray/0.1",
    },
  });
  const text = await res.text();
  if (!res.ok) {
    throw new Error(`Copilot HTTP ${res.status}`);
  }
  return parseCopilotUser(JSON.parse(text) as unknown);
}

function asObject(v: unknown): Record<string, unknown> | null {
  if (v && typeof v === "object" && !Array.isArray(v)) {
    return v as Record<string, unknown>;
  }
  return null;
}
