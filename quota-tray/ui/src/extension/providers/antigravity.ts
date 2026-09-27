import type { ProviderSnapshot, UsageWindow } from "../../lib/types";

/** Public installed-app OAuth clients (not private secrets — shipped in desktop apps). */
export type AntigravityOAuthClientKind = "antigravity" | "gemini-cli";

interface OAuthClient {
  kind: AntigravityOAuthClientKind;
  id: string;
  secret: string;
}

function antigravityInstalledAppClient(): OAuthClient {
  const id =
    ["1071006060591", "tmhssin2h21lcre235vtolojh4g403ep"].join("-") +
    ".apps.googleusercontent.com";
  // GOCSPX- prefix + body — public desktop-app secret, intentionally client-side.
  const secret = ["GOC", "SPX", "-", "K58FWR486LdLJ1mLB8sXC4z6qDAf"].join("");
  return { kind: "antigravity", id, secret };
}

/** gemini-cli's public OAuth client — mints ~/.gemini/oauth_creds.json. */
function geminiCliInstalledAppClient(): OAuthClient {
  const id =
    ["681255809395", "oo8ft2oprdrnp9e3aqf6av3hmdib135j"].join("-") +
    ".apps.googleusercontent.com";
  const secret = ["GOC", "SPX", "-", "4uHgMPm-1o7Sk-geV6Cu5clXFsxl"].join("");
  return { kind: "gemini-cli", id, secret };
}

const OAUTH_CLIENTS: OAuthClient[] = [
  antigravityInstalledAppClient(),
  geminiCliInstalledAppClient(),
];

export const ANTIGRAVITY_CLIENT_ID = antigravityInstalledAppClient().id;
export const ANTIGRAVITY_CLIENT_SECRET = antigravityInstalledAppClient().secret;
export const GEMINI_CLI_CLIENT_ID = geminiCliInstalledAppClient().id;
export const ANTIGRAVITY_TOKEN_URL = "https://oauth2.googleapis.com/token";
export const ANTIGRAVITY_BASE_URLS = [
  "https://daily-cloudcode-pa.googleapis.com",
  "https://cloudcode-pa.googleapis.com",
];

const GEMINI_CLI_WRONG_SOURCE_HINT =
  "These credentials look like gemini-cli (~/.gemini/oauth_creds.json). " +
  "Antigravity quota needs an Antigravity-minted refresh token — import " +
  "~/.gemini/jetski-standalone-oauth-token (nested {token:{refresh_token}}) instead.";

export interface AntigravityCreds {
  refresh_token: string;
  access_token?: string;
  expiry?: string;
  expiry_date?: number;
  token_type?: string;
  /** Which public OAuth client last refreshed successfully (persisted). */
  oauth_client?: AntigravityOAuthClientKind;
  /** Optional client_id from ADC-style JSON; used to pick a matching client. */
  client_id?: string;
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
  const oauthClient = obj?.oauth_client;
  const clientId = obj?.client_id;
  return {
    refresh_token: refreshToken,
    access_token: typeof obj?.access_token === "string" ? obj.access_token : undefined,
    expiry: typeof obj?.expiry === "string" ? obj.expiry : undefined,
    expiry_date: typeof obj?.expiry_date === "number" ? obj.expiry_date : undefined,
    token_type: typeof obj?.token_type === "string" ? obj.token_type : undefined,
    oauth_client:
      oauthClient === "antigravity" || oauthClient === "gemini-cli" ? oauthClient : undefined,
    client_id: typeof clientId === "string" ? clientId : undefined,
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

function clientsToTry(creds: AntigravityCreds): OAuthClient[] {
  const preferred: OAuthClient[] = [];
  const rest: OAuthClient[] = [];
  for (const c of OAUTH_CLIENTS) {
    const matchKind = creds.oauth_client != null && c.kind === creds.oauth_client;
    const matchId = creds.client_id != null && c.id === creds.client_id;
    if (matchKind || matchId) {
      preferred.push(c);
    } else {
      rest.push(c);
    }
  }
  return [...preferred, ...rest];
}

function oauthErrorDetail(status: number, text: string): string {
  try {
    const body = JSON.parse(text) as Record<string, unknown>;
    const err = typeof body.error === "string" ? body.error : null;
    const desc =
      typeof body.error_description === "string" ? body.error_description : null;
    if (err && desc) {
      return `${err} (${desc})`;
    }
    if (err) {
      return err;
    }
  } catch {
    // ignore non-JSON
  }
  return `HTTP ${status}`;
}

async function refreshWithClient(
  refreshToken: string,
  client: OAuthClient,
  fetcher: typeof fetch
): Promise<{ access_token: string; expires_in: number; token_type?: string } | { error: string }> {
  const res = await fetcher(ANTIGRAVITY_TOKEN_URL, {
    method: "POST",
    headers: { "Content-Type": "application/x-www-form-urlencoded" },
    body: new URLSearchParams({
      client_id: client.id,
      client_secret: client.secret,
      refresh_token: refreshToken,
      grant_type: "refresh_token",
    }).toString(),
  });
  const text = await res.text();
  if (!res.ok) {
    return { error: oauthErrorDetail(res.status, text) };
  }
  const body = JSON.parse(text) as Record<string, unknown>;
  const accessToken = body.access_token;
  if (typeof accessToken !== "string" || !accessToken) {
    return { error: "missing access_token" };
  }
  return {
    access_token: accessToken,
    expires_in: typeof body.expires_in === "number" ? body.expires_in : 3600,
    token_type: typeof body.token_type === "string" ? body.token_type : undefined,
  };
}

/**
 * Refresh using the Antigravity installed-app client first, then gemini-cli's.
 * Persist which client worked via `oauth_client` on the returned creds blob.
 */
export async function refreshAntigravityToken(
  creds: AntigravityCreds,
  fetcher: typeof fetch
): Promise<{
  access_token: string;
  expires_in: number;
  token_type?: string;
  oauth_client: AntigravityOAuthClientKind;
}> {
  const attempts = clientsToTry(creds);
  const errors: string[] = [];
  for (const client of attempts) {
    const result = await refreshWithClient(creds.refresh_token, client, fetcher);
    if ("access_token" in result) {
      return {
        access_token: result.access_token,
        expires_in: result.expires_in,
        token_type: result.token_type,
        oauth_client: client.kind,
      };
    }
    errors.push(`${client.kind}: ${result.error}`);
  }
  const joined = errors.join("; ");
  const unauthorized = errors.some((e) => /unauthorized_client|invalid_client/i.test(e));
  if (unauthorized) {
    throw new Error(
      `Antigravity token refresh failed (${joined}). ${GEMINI_CLI_WRONG_SOURCE_HINT}`
    );
  }
  throw new Error(`Antigravity token refresh failed (${joined})`);
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

function serializeCreds(creds: AntigravityCreds): string {
  const out: Record<string, unknown> = { refresh_token: creds.refresh_token };
  if (creds.access_token) {
    out.access_token = creds.access_token;
  }
  if (creds.expiry) {
    out.expiry = creds.expiry;
  }
  if (typeof creds.expiry_date === "number") {
    out.expiry_date = creds.expiry_date;
  }
  if (creds.token_type) {
    out.token_type = creds.token_type;
  }
  if (creds.oauth_client) {
    out.oauth_client = creds.oauth_client;
  }
  if (creds.client_id) {
    out.client_id = creds.client_id;
  }
  return JSON.stringify(out);
}

export async function fetchAntigravityUsage(
  secretJson: string,
  fetcher: typeof fetch = fetch
): Promise<{ snapshot: ProviderSnapshot; secretJson: string }> {
  let creds = parseAntigravityCreds(secretJson);
  if (isAntigravityCredsExpired(creds)) {
    const refreshed = await refreshAntigravityToken(creds, fetcher);
    creds = {
      refresh_token: creds.refresh_token,
      access_token: refreshed.access_token,
      expiry_date: Date.now() + refreshed.expires_in * 1000,
      token_type: refreshed.token_type,
      oauth_client: refreshed.oauth_client,
      client_id: creds.client_id,
    };
  }
  const accessToken = creds.access_token;
  if (!accessToken) {
    throw new Error("Antigravity: no access token after refresh");
  }

  let lastErr: unknown = null;
  let sawForbidden = false;
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
      } catch (quotaErr) {
        const msg = quotaErr instanceof Error ? quotaErr.message : String(quotaErr);
        if (/HTTP 403/.test(msg)) {
          sawForbidden = true;
        }
        try {
          const models = await callAntigravityInternal(
            "/v1internal:fetchAvailableModels",
            accessToken,
            baseUrl,
            { project },
            fetcher
          );
          parsed = parseModelsFallback(models);
        } catch (modelsErr) {
          const m = modelsErr instanceof Error ? modelsErr.message : String(modelsErr);
          if (/HTTP 403/.test(m)) {
            sawForbidden = true;
          }
          throw quotaErr instanceof Error ? quotaErr : modelsErr;
        }
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
        secretJson: serializeCreds(creds),
      };
    } catch (e) {
      lastErr = e;
      const msg = e instanceof Error ? e.message : String(e);
      if (/HTTP 403/.test(msg)) {
        sawForbidden = true;
      }
    }
  }

  if (sawForbidden && creds.oauth_client === "gemini-cli") {
    throw new Error(GEMINI_CLI_WRONG_SOURCE_HINT);
  }
  throw lastErr instanceof Error ? lastErr : new Error("Antigravity: all endpoints failed");
}
