import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { TrayState } from "./types";

export async function fetchTrayState(): Promise<TrayState> {
  return invoke<TrayState>("get_tray_state");
}

export function onTrayStateUpdated(cb: () => void): Promise<() => void> {
  return listen("tray-state-updated", () => cb()).then((unlisten) => unlisten);
}
