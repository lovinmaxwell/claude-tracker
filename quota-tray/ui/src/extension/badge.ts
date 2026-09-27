import type { TrayState } from "../lib/types";

export async function updateActionBadge(state: TrayState): Promise<void> {
  const fill = state.shared_mascot_fill;
  if (fill == null || Number.isNaN(fill)) {
    await chrome.action.setBadgeText({ text: "" });
    return;
  }
  const n = Math.round(fill);
  await chrome.action.setBadgeText({ text: String(n) });
  const color = n >= 75 ? "#d94b34" : n >= 50 ? "#c96442" : "#5a7a6a";
  await chrome.action.setBadgeBackgroundColor({ color });
}
