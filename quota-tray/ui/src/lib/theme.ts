/** Theme preference: System follows OS; Light/Dark force appearance. */
export type ThemeMode = "system" | "light" | "dark";

export type PaletteId = "studio" | "cyberpunk" | "tokyonight" | "matrix";

export interface PaletteOption {
  id: PaletteId;
  label: string;
  description: string;
  swatches: string[];
}

export const PALETTE_OPTIONS: PaletteOption[] = [
  {
    id: "studio",
    label: "Studio Terracotta",
    description: "Artisanal cream, terracotta, and sage. Calm and natural.",
    swatches: ["#c96442", "#5a7a6a", "#d94b34"],
  },
  {
    id: "cyberpunk",
    label: "Neon Terminal",
    description: "Luminescent electric amber, laser cyan, and hot crimson glow.",
    swatches: ["#ff7a45", "#00f5d4", "#ff0055"],
  },
  {
    id: "tokyonight",
    label: "Tokyo Night",
    description: "Twilight indigo with soft peach, sky blue, and synth pink.",
    swatches: ["#ff9e64", "#7aa2f7", "#f7768e"],
  },
  {
    id: "matrix",
    label: "Matrix Emerald",
    description: "Phosphor green, mint highlights, and signal gold warning blocks.",
    swatches: ["#66bb6a", "#00e676", "#ffd600"],
  },
];

const STORAGE_THEME_KEY = "quota-tray-theme";
const STORAGE_PALETTE_KEY = "quota-tray-palette";

export function readThemeMode(): ThemeMode {
  try {
    const v = localStorage.getItem(STORAGE_THEME_KEY);
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
    localStorage.setItem(STORAGE_THEME_KEY, mode);
  } catch {
    /* ignore */
  }
}

export function initTheme(): ThemeMode {
  const mode = readThemeMode();
  applyThemeMode(mode);
  return mode;
}

export function readPalette(): PaletteId {
  try {
    const v = localStorage.getItem(STORAGE_PALETTE_KEY);
    if (v === "studio" || v === "cyberpunk" || v === "tokyonight" || v === "matrix") {
      return v;
    }
  } catch {
    /* ignore */
  }
  return "studio";
}

export function applyPalette(id: PaletteId): void {
  const root = document.documentElement;
  root.setAttribute("data-palette", id);
  try {
    localStorage.setItem(STORAGE_PALETTE_KEY, id);
  } catch {
    /* ignore */
  }
}

export function initPalette(): PaletteId {
  const palette = readPalette();
  applyPalette(palette);
  return palette;
}

export function cyclePalette(current: PaletteId): PaletteId {
  const order: PaletteId[] = ["studio", "cyberpunk", "tokyonight", "matrix"];
  const nextIdx = (order.indexOf(current) + 1) % order.length;
  return order[nextIdx];
}
