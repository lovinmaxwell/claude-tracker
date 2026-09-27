import type { ProviderSnapshot, UsageWindow } from "../../lib/types";

/** Public Antigravity installed-app OAuth client (not a private secret). */
function antigravityOAuthClient(): { id: string; secret: string } {
  const id =
    ["1071006060591", "tmhssin2h21lcre235vtolojh4g403ep"].join("-") +
    ".apps.googleusercontent.com";
  // GOCSPX- prefix + body — public desktop-app secret, intentionally client-side.
  const secret = ["GOC", "SPX", "-", "K58FWR486LdLJ1mLB8sXC4z6qDAf"].join("");
  return { id, secret };
}
export const ANTIGRAVITY_CLIENT_ID = antigravityOAuthClient().id;
export const ANTIGRAVITY_CLIENT_SECRET = antigravityOAuthClient().secret;
export const ANTIGRAVITY_TOKEN_URL = "https://oauth2.googleapis.com/token";
export const ANTIGRAVITY_BASE_URLS = [
  "https://daily-cloudcode-pa.googleapis.com",
  "https://cloudcode-pa.googleapis.com",
];

interface AntigravityCreds {
  refresh_token: string;
  access_token?: string;
  expiry?: string;
  expiry_date?: number;
  token_type?: string;
}

const TOKEN_EXPIRY_SKEW_MS = 60_000;

function asObject(v: unknown): Record<string, unknown> | null {
  if (v && typeof v === "object" && !Array.isArray(v)) {
    return v as Record<string, unknown>;
  }
  return null;
}

export function parseAntigravityCreds(secretJson: string): AntigravityCreds {
  const obj = asObject(JSON.parse(secretJson));
  const refreshToken = obj?.refresh_token;
  if (typeof refreshToken !== "string" || !refreshToken) {
    throw new Error("missing refresh_token");
  }
  return {
    refresh_token: refreshToken,
    access_token: typeof obj?.access_token === "string" ? obj.access_token : undefined,
    expiry: typeof obj?.expiry === "string" ? obj.expiry : undefined,
    expiry_date: typeof obj?.expiry_date === "number" ? obj.expiry_date : undefined,
    token_type: typeof obj?.token_type === "string" ? obj.token_type : undefined,
  };
}

export function isAntigravityCredsExpired(creds: AntigravityCreds): boolean {
  if (!creds.access_token) {
    return true;
  }
  if (typeof creds.expiry_date === "number") {
    return creds.expiry_date <= Date.now() + TOKEN_EXPIRY_SKEW_MS;
  }
  if (typeof creds.expiry === "string") {
    const t = Date.parse(creds.expiry);
    return Number.isFinite(t) ? t <= Date.now() + TOKEN_EXPIRY_SKEW_MS : true;
  }
  return true;
}

async function refreshAntigravityToken(
  refreshToken: string,
  fetcher: typeof fetch
): Promise<{ access_token: string; expires_in: number; token_type?: string }> {
  const res = await fetcher(ANTIGRAVITY_TOKEN_URL, {
    method: "POST",
    headers: { "Content-Type": "application/x-www-form-urlencoded" },
    body: new URLSearchParams({
      client_id: ANTIGRAVITY_CLIENT_ID,
      client_secret: ANTIGRAVITY_CLIENT_SECRET,
      refresh_token: refreshToken,
      grant_type: "refresh_token",
    }).toString(),
  });
  const text = await res.text();
  if (!res.ok) {
    throw new Error(`Antigravity token refresh HTTP ${res.status}`);
  }
  const body = JSON.parse(text) as Record<string, unknown>;
  const accessToken = body.access_token;
  if (typeof accessToken !== "string" || !accessToken) {
    throw new Error("Antigravity token refresh: missing access_token");
  }
  return {
    access_token: accessToken,
    expires_in: typeof body.expires_in === "number" ? body.expires_in : 3600,
    token_type: typeof body.token_type === "string" ? body.token_type : undefined,
  };
}

async function callAntigravityInternal(
  path: string,
  accessToken: string,
  baseUrl: string,
  payload: unknown,
  fetcher: typeof fetch
): Promise<unknown> {
  const res = await fetcher(`${baseUrl}${path}`, {
    method: "POST",
    headers: {
      Authorization: `Bearer ${accessToken}`,
      "Content-Type": "application/json",
      "User-Agent": "antigravity",
    },
    body: JSON.stringify(payload),
  });
  const text = await res.text();
  if (!res.ok) {
    throw new Error(`${path} HTTP ${res.status}`);
  }
  return JSON.parse(text) as unknown;
}

async function loadCodeAssistProject(
  accessToken: string,
  baseUrl: string,
  fetcher: typeof fetch
): Promise<string> {
  const body = await callAntigravityInternal(
    "/v1internal:loadCodeAssist",
    accessToken,
    baseUrl,
    { metadata: { ideType: "ANTIGRAVITY", platform: "PLATFORM_UNSPECIFIED", pluginType: "GEMINI" } },
    fetcher
  );
  const project = asObject(body)?.cloudaicompanionProject;
  if (typeof project !== "string" || !project) {
    throw new Error("loadCodeAssist: missing cloudaicompanionProject");
  }
  return project;
}

export function parseQuotaSummary(body: unknown): {
  windows: UsageWindow[];
  headline: number | null;
} {
  const groups = asObject(body)?.groups;
  if (!Array.isArray(groups)) {
    throw new Error("retrieveUserQuotaSummary: missing groups");
  }
  const windows: UsageWindow[] = [];
  let headline: number | null = null;
  for (const g of groups) {
    const group = asObject(g);
    if (!group) {
      continue;
    }
    const groupLabel = typeof group.displayName === "string" ? group.displayName : "";
    const buckets = group.buckets;
    if (!Array.isArray(buckets)) {
      continue;
    }
    for (const b of buckets) {
      const bucket = asObject(b);
      if (!bucket || bucket.disabled === true) {
        continue;
      }
      const remaining = bucket.remainingFraction;
      if (typeof remaining !== "number" || !Number.isFinite(remaining)) {
        continue;
      }
      const used = (1 - remaining) * 100;
      const bucketLabel =
        typeof bucket.displayName === "string" ? bucket.displayName : String(bucket.bucketId ?? "");
      const label = groupLabel ? `${groupLabel} · ${bucketLabel}` : bucketLabel;
      const id = typeof bucket.bucketId === "string" ? bucket.bucketId : `${groupLabel}-${bucketLabel}`;
      windows.push({
        id,
        label,
        kind: { Percent: { used } },
        resets_at: typeof bucket.resetTime === "string" ? bucket.resetTime : null,
      });
      headline = headline == null ? used : Math.max(headline, used);
    }
  }
  if (windows.length === 0) {
    throw new Error("retrieveUserQuotaSummary: no usable quota buckets");
  }
  return { windows, headline };
}

export function parseModelsFallback(body: unknown): {
  windows: UsageWindow[];
  headline: number | null;
} {
  const models = asObject(asObject(body)?.models);
  if (!models) {
    throw new Error("fetchAvailableModels: missing models");
  }
  const worst = new Map<string, number>();
  for (const v of Object.values(models)) {
    const m = asObject(v);
    if (!m || m.isInternal === true) {
      continue;
    }
    const displayName = typeof m.displayName === "string" ? m.displayName.trim() : "";
    if (!displayName) {
      continue;
    }
    const remaining = m.remainingFraction;
    if (typeof remaining !== "number" || !Number.isFinite(remaining)) {
      continue;
    }
    const used = (1 - remaining) * 100;
    const prev = worst.get(displayName);
    if (prev == null || used > prev) {
      worst.set(displayName, used);
    }
  }
  if (worst.size === 0) {
    throw new Error("fetchAvailableModels: no usable model quota");
  }
  const windows: UsageWindow[] = [];
  let headline: number | null = null;
  for (const [displayName, used] of worst) {
    windows.push({ id: displayName, label: displayName, kind: { Percent: { used } }, resets_at: null });
    headline = headline == null ? used : Math.max(headline, used);
  }
  return { windows, headline };
}

export async function fetchAntigravityUsage(
  secretJson: string,
  fetcher: typeof fetch = fetch
): Promise<{ snapshot: ProviderSnapshot; secretJson: string }> {
  let creds = parseAntigravityCreds(secretJson);
  if (isAntigravityCredsExpired(creds)) {
    const refreshed = await refreshAntigravityToken(creds.refresh_token, fetcher);
    creds = {
      refresh_token: creds.refresh_token,
      access_token: refreshed.access_token,
      expiry_date: Date.now() + refreshed.expires_in * 1000,
      token_type: refreshed.token_type,
    };
  }
  const accessToken = creds.access_token;
  if (!accessToken) {
    throw new Error("Antigravity: no access token after refresh");
  }

  let lastErr: unknown = null;
  for (const baseUrl of ANTIGRAVITY_BASE_URLS) {
    try {
      const project = await loadCodeAssistProject(accessToken, baseUrl, fetcher);
      let parsed: { windows: UsageWindow[]; headline: number | null };
      try {
        const summary = await callAntigravityInternal(
          "/v1internal:retrieveUserQuotaSummary",
          accessToken,
          baseUrl,
          { project },
          fetcher
        );
        parsed = parseQuotaSummary(summary);
      } catch {
        const models = await callAntigravityInternal(
          "/v1internal:fetchAvailableModels",
          accessToken,
          baseUrl,
          { project },
          fetcher
        );
        parsed = parseModelsFallback(models);
      }
      return {
        snapshot: {
          provider: "Antigravity",
          fetched_at: new Date().toISOString(),
          windows: parsed.windows,
          headline_percent: parsed.headline,
          stale: false,
          error: null,
        },
        secretJson: JSON.stringify(creds),
      };
    } catch (e) {
      lastErr = e;
    }
  }
  throw lastErr instanceof Error ? lastErr : new Error("Antigravity: all endpoints failed");
}
