/** True inside the Tauri webview; false in a plain browser (dev preview). */
export function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

/**
 * The native side owns hover detection (WKWebView stops reporting pointer
 * events once the app loses focus), so the widget listens for its verdict.
 */
export async function onWidgetHover(cb: (hovered: boolean) => void): Promise<() => void> {
  const { listen } = await import("@tauri-apps/api/event");
  return listen<boolean>("widget-hover", (e) => cb(e.payload));
}

export async function hideWidget(): Promise<void> {
  const { invoke } = await import("@tauri-apps/api/core");
  await invoke("set_widget_visible", { visible: false });
}
