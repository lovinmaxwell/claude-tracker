import { describe, expect, it, beforeEach, vi } from "vitest";
import { applyThemeMode, readThemeMode } from "./theme";

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
});
