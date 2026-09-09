import { sanitizeConfig, type AppConfig } from "./config";
import { isChromeExtension } from "./platform";

export async function fetchConfig(): Promise<AppConfig> {
  if (isChromeExtension()) {
    const res = (await chrome.runtime.sendMessage({
      type: "get_config",
    })) as AppConfig & { error?: string };
    if (res && "error" in res && res.error) {
      throw new Error(res.error);
    }
    return sanitizeConfig(res);
  }
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<AppConfig>("get_config");
}

export async function saveConfig(config: AppConfig): Promise<AppConfig> {
  const next = sanitizeConfig(config);
  if (isChromeExtension()) {
    const res = (await chrome.runtime.sendMessage({
      type: "set_config",
      config: next,
    })) as { config?: AppConfig; error?: string };
    if (res?.error) {
      throw new Error(res.error);
    }
    return sanitizeConfig(res.config ?? next);
  }
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<AppConfig>("set_config", { config: next });
}

export type ConnectionKind = "missing" | "imported" | "browser";
export type ConnectionMap = Record<"Claude" | "Cursor" | "Copilot", ConnectionKind>;

export async function fetchConnections(): Promise<ConnectionMap> {
  if (!isChromeExtension()) {
    return { Claude: "missing", Cursor: "missing", Copilot: "missing" };
  }
  const res = (await chrome.runtime.sendMessage({
    type: "get_connections",
  })) as ConnectionMap & { error?: string };
  if (res && "error" in res && res.error) {
    throw new Error(res.error);
  }
  return res;
}

export async function importSecret(
  provider: "Claude" | "Cursor" | "Copilot",
  text: string
): Promise<void> {
  const res = (await chrome.runtime.sendMessage({
    type: "import_secret",
    provider,
    text,
  })) as { error?: string };
  if (res?.error) {
    throw new Error(res.error);
  }
}

export async function clearSecret(
  provider: "Claude" | "Cursor" | "Copilot"
): Promise<void> {
  const res = (await chrome.runtime.sendMessage({
    type: "clear_secret",
    provider,
  })) as { error?: string };
  if (res?.error) {
    throw new Error(res.error);
  }
}
