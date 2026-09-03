/** Theme preference: System follows OS; Light/Dark force appearance. */
export type ThemeMode = "system" | "light" | "dark";

const STORAGE_KEY = "quota-tray-theme";

export function readThemeMode(): ThemeMode {
  try {
    const v = localStorage.getItem(STORAGE_KEY);
    if (v === "light" || v === "dark" || v === "system") {
      return v;
    }
  } catch {
    /* ignore */
  }
  return "system";
}

export function applyThemeMode(mode: ThemeMode): void {
  const root = document.documentElement;
  if (mode === "system") {
    root.removeAttribute("data-theme");
  } else {
    root.setAttribute("data-theme", mode);
  }
  try {
    localStorage.setItem(STORAGE_KEY, mode);
  } catch {
    /* ignore */
  }
}

export function initTheme(): ThemeMode {
  const mode = readThemeMode();
  applyThemeMode(mode);
  return mode;
}
