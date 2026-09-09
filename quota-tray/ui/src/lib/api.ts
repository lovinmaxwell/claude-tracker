import type { TrayState } from "./types";
import { isChromeExtension } from "./platform";

type ChromeResponse = TrayState | { error?: string };

export async function fetchTrayState(): Promise<TrayState> {
  if (isChromeExtension()) {
    const res = (await chrome.runtime.sendMessage({
      type: "get_tray_state",
    })) as ChromeResponse | undefined;
    if (!res) {
      throw new Error("extension background unavailable");
    }
    if (res && "error" in res && res.error) {
      throw new Error(res.error);
    }
    return res as TrayState;
  }
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<TrayState>("get_tray_state");
}

export function onTrayStateUpdated(cb: () => void): Promise<() => void> {
  if (isChromeExtension()) {
    const listener: Parameters<typeof chrome.storage.onChanged.addListener>[0] = (
      changes,
      area
    ) => {
      if (area === "local" && changes["quota-tray-state"]) {
        cb();
      }
    };
    chrome.storage.onChanged.addListener(listener);
    return Promise.resolve(() => {
      chrome.storage.onChanged.removeListener(listener);
    });
  }
  return import("@tauri-apps/api/event").then(({ listen }) =>
    listen("tray-state-updated", () => cb()).then((unlisten) => unlisten)
  );
}

export async function refreshNow(): Promise<TrayState> {
  if (isChromeExtension()) {
    const res = (await chrome.runtime.sendMessage({
      type: "refresh_now",
    })) as ChromeResponse | undefined;
    if (!res) {
      throw new Error("extension background unavailable");
    }
    if (res && "error" in res && res.error) {
      throw new Error(res.error);
    }
    return res as TrayState;
  }
  return fetchTrayState();
}
