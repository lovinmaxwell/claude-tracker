import { defaultConfig, sanitizeConfig, type AppConfig } from "../lib/config";
import type { ProviderId, TrayState } from "../lib/types";

const KEY_CONFIG = "quota-tray-config";
const KEY_STATE = "quota-tray-state";
const KEY_MASCOT = "quota-tray-last-mascot";
const KEY_SECRETS = "quota-tray-secrets";

type Secrets = Partial<Record<ProviderId, string>>;

export async function loadConfig(): Promise<AppConfig> {
  const raw = await chrome.storage.local.get(KEY_CONFIG);
  const stored = raw[KEY_CONFIG] as AppConfig | undefined;
  if (!stored) {
    return defaultConfig();
  }
  return sanitizeConfig(stored);
}

export async function saveConfig(config: AppConfig): Promise<AppConfig> {
  const next = sanitizeConfig(config);
  await chrome.storage.local.set({ [KEY_CONFIG]: next });
  return next;
}

export async function loadTrayState(): Promise<TrayState | null> {
  const raw = await chrome.storage.local.get(KEY_STATE);
  return (raw[KEY_STATE] as TrayState | undefined) ?? null;
}

export async function saveTrayState(state: TrayState): Promise<void> {
  await chrome.storage.local.set({ [KEY_STATE]: state });
}

export async function loadLastMascot(): Promise<number | null> {
  const raw = await chrome.storage.local.get(KEY_MASCOT);
  const v = raw[KEY_MASCOT];
  return typeof v === "number" && Number.isFinite(v) ? v : null;
}

export async function saveLastMascot(value: number | null): Promise<void> {
  await chrome.storage.local.set({ [KEY_MASCOT]: value });
}

export async function loadSecrets(): Promise<Secrets> {
  const raw = await chrome.storage.local.get(KEY_SECRETS);
  return (raw[KEY_SECRETS] as Secrets | undefined) ?? {};
}

export async function saveSecret(provider: ProviderId, token: string): Promise<void> {
  const secrets = await loadSecrets();
  secrets[provider] = token;
  await chrome.storage.local.set({ [KEY_SECRETS]: secrets });
}

export async function clearSecret(provider: ProviderId): Promise<void> {
  const secrets = await loadSecrets();
  delete secrets[provider];
  await chrome.storage.local.set({ [KEY_SECRETS]: secrets });
}

export type ConnectionKind = "missing" | "imported" | "browser";

export type ConnectionMap = Record<"Claude" | "Cursor" | "Copilot", ConnectionKind>;
