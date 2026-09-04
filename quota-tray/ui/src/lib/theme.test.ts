import { describe, expect, it, beforeEach, vi } from "vitest";
import {
  applyThemeMode,
  readThemeMode,
  readPalette,
  applyPalette,
  initPalette,
  cyclePalette,
  PALETTE_OPTIONS,
} from "./theme";

describe("theme", () => {
  const store = new Map<string, string>();
  const attrs = new Map<string, string>();

  beforeEach(() => {
    store.clear();
    attrs.clear();
    vi.stubGlobal("localStorage", {
      getItem: (k: string) => store.get(k) ?? null,
      setItem: (k: string, v: string) => {
        store.set(k, v);
      },
      removeItem: (k: string) => {
        store.delete(k);
      },
      clear: () => store.clear(),
    });
    vi.stubGlobal("document", {
      documentElement: {
        setAttribute: (k: string, v: string) => attrs.set(k, v),
        removeAttribute: (k: string) => attrs.delete(k),
        getAttribute: (k: string) => attrs.get(k) ?? null,
        hasAttribute: (k: string) => attrs.has(k),
      },
    });
  });

  it("defaults to system", () => {
    expect(readThemeMode()).toBe("system");
  });

  it("persists light and dark", () => {
    applyThemeMode("dark");
    expect(attrs.get("data-theme")).toBe("dark");
    expect(readThemeMode()).toBe("dark");

    applyThemeMode("light");
    expect(attrs.get("data-theme")).toBe("light");
    expect(readThemeMode()).toBe("light");

    applyThemeMode("system");
    expect(attrs.has("data-theme")).toBe(false);
    expect(readThemeMode()).toBe("system");
  });

  it("defaults palette to studio", () => {
    expect(readPalette()).toBe("studio");
  });

  it("persists and applies palette attribute", () => {
    applyPalette("cyberpunk");
    expect(attrs.get("data-palette")).toBe("cyberpunk");
    expect(readPalette()).toBe("cyberpunk");

    applyPalette("tokyonight");
    expect(attrs.get("data-palette")).toBe("tokyonight");
    expect(readPalette()).toBe("tokyonight");

    applyPalette("matrix");
    expect(attrs.get("data-palette")).toBe("matrix");
    expect(readPalette()).toBe("matrix");
  });

  it("cycles through all palettes in order", () => {
    expect(cyclePalette("studio")).toBe("cyberpunk");
    expect(cyclePalette("cyberpunk")).toBe("tokyonight");
    expect(cyclePalette("tokyonight")).toBe("matrix");
    expect(cyclePalette("matrix")).toBe("studio");
  });

  it("exposes all 4 palette options with swatches", () => {
    expect(PALETTE_OPTIONS.length).toBe(4);
    expect(PALETTE_OPTIONS.map((p) => p.id)).toEqual([
      "studio",
      "cyberpunk",
      "tokyonight",
      "matrix",
    ]);
  });
});
