import type { ProviderId } from "../lib/types";

export function parseClaudeSecret(raw: string): string {
  const trimmed = raw.trim();
  if (!trimmed) {
    throw new Error("empty Claude credentials");
  }
  if (trimmed.startsWith("{")) {
    const root = JSON.parse(trimmed) as {
      claudeAiOauth?: { accessToken?: string; expiresAt?: number };
    };
    const token = root.claudeAiOauth?.accessToken?.trim();
    if (!token) {
      throw new Error("missing claudeAiOauth.accessToken");
    }
    const expires = root.claudeAiOauth?.expiresAt;
    if (typeof expires === "number" && expires > 0 && expires <= Date.now()) {
      throw new Error("Claude OAuth token expired");
    }
    return token;
  }
  return trimmed;
}

export function parseCopilotSecret(raw: string): string {
  const trimmed = raw.trim();
  if (!trimmed) {
    throw new Error("empty Copilot credentials");
  }
  if (trimmed.startsWith("{")) {
    const v = JSON.parse(trimmed) as Record<string, { oauth_token?: string }>;
    for (const entry of Object.values(v)) {
      const token = entry?.oauth_token?.trim();
      if (token) {
        return token;
      }
    }
    throw new Error("no oauth_token in github-copilot credential json");
  }
  return trimmed;
}

export function parseCursorSecret(raw: string): string {
  const trimmed = raw.trim();
  if (!trimmed) {
    throw new Error("empty Cursor token");
  }
  return extractCursorTokenFromCookie(trimmed) ?? trimmed;
}

/** WorkosCursorSessionToken is `{userId}::{accessToken}` (URL-encoded `::` as `%3A%3A`). */
export function extractCursorTokenFromCookie(value: string): string | null {
  if (!value) {
    return null;
  }
  let decoded = value;
  try {
    decoded = decodeURIComponent(value);
  } catch {
    decoded = value;
  }
  const sep = "::";
  const idx = decoded.indexOf(sep);
  if (idx >= 0) {
    const token = decoded.slice(idx + sep.length).trim();
    return token.length > 0 ? token : null;
  }
  const jwtish = decoded.trim();
  return jwtish.length > 0 ? jwtish : null;
}

/**
 * Accepts:
 *  a) full oauth JSON: {access_token, refresh_token, expiry?/expiry_date?, token_type?}
 *  b) nested keychain/jetski shape: {token: {access_token, refresh_token, ...}, auth_method?}
 *  c) gemini oauth_creds.json shape (flat, with refresh_token)
 *  d) raw refresh_token string
 * Returns a JSON string containing at least refresh_token, for storage and later refresh.
 */
export function parseAntigravitySecret(raw: string): string {
  const trimmed = raw.trim();
  if (!trimmed) {
    throw new Error("empty Antigravity credentials");
  }
  if (!trimmed.startsWith("{")) {
    return JSON.stringify({ refresh_token: trimmed });
  }
  const root = JSON.parse(trimmed) as Record<string, unknown>;
  const nested =
    root.token && typeof root.token === "object" && !Array.isArray(root.token)
      ? (root.token as Record<string, unknown>)
      : null;
  const src = nested ?? root;
  const refreshToken = src.refresh_token;
  if (typeof refreshToken !== "string" || !refreshToken) {
    throw new Error("no refresh_token in Antigravity credentials");
  }
  const out: Record<string, unknown> = { refresh_token: refreshToken };
  if (typeof src.access_token === "string") {
    out.access_token = src.access_token;
  }
  if (typeof src.expiry === "string") {
    out.expiry = src.expiry;
  }
  if (typeof src.expiry_date === "number") {
    out.expiry_date = src.expiry_date;
  }
  if (typeof src.token_type === "string") {
    out.token_type = src.token_type;
  }
  // Preserve client hints when present (ADC JSON or prior refresh).
  if (src.oauth_client === "antigravity" || src.oauth_client === "gemini-cli") {
    out.oauth_client = src.oauth_client;
  }
  if (typeof src.client_id === "string" && src.client_id) {
    out.client_id = src.client_id;
  } else if (typeof root.client_id === "string" && root.client_id) {
    out.client_id = root.client_id;
  }
  return JSON.stringify(out);
}

export function parseImportedSecret(provider: ProviderId, raw: string): string {
  switch (provider) {
    case "Claude":
      return parseClaudeSecret(raw);
    case "Cursor":
      return parseCursorSecret(raw);
    case "Copilot":
      return parseCopilotSecret(raw);
    case "Antigravity":
      return parseAntigravitySecret(raw);
    default:
      throw new Error(`unsupported provider ${provider}`);
  }
}
