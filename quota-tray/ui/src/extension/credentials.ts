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

export function parseImportedSecret(provider: ProviderId, raw: string): string {
  switch (provider) {
    case "Claude":
      return parseClaudeSecret(raw);
    case "Cursor":
      return parseCursorSecret(raw);
    case "Copilot":
      return parseCopilotSecret(raw);
    default:
      throw new Error(`unsupported provider ${provider}`);
  }
}
