export function isChromeExtension(): boolean {
  if (import.meta.env.QUOTA_TRAY_PLATFORM === "chrome") {
    return true;
  }
  return (
    typeof chrome !== "undefined" &&
    typeof chrome.runtime !== "undefined" &&
    typeof chrome.runtime.id === "string" &&
    chrome.runtime.id.length > 0
  );
}

export async function hidePanel(): Promise<void> {
  if (isChromeExtension()) {
    window.close();
    return;
  }
  const { getCurrentWindow } = await import("@tauri-apps/api/window");
  await getCurrentWindow().hide();
}

export function emptyProvidersBody(): string {
  if (isChromeExtension()) {
    return "Turn on a provider in Settings. Cursor can use your cursor.com login in this browser.";
  }
  return "Turn on Claude, Cursor, or Copilot in Settings. Tokens stay on this Mac.";
}
