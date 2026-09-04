# Quota Tray: Precision Segmented HUD & Multi-Palette Theming Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Transform the Quota Tray menu bar app into a high-density, engineer-grade Precision Segmented HUD with 12-segment LED fill meters, compact HUD topbar, and a selectable 4-palette theme engine (Studio, Cyberpunk, Tokyo Night, Matrix).

**Architecture:** A dual-axis theming system (mode: system/light/dark, palette: studio/cyberpunk/tokyonight/matrix) managed in `theme.ts` via `localStorage` and `data-palette` attributes on `<html>`, styled via CSS custom properties in `app.css`. The continuous meter in `ProviderRow.svelte` is replaced by a 12-segment discrete LED bar with warning thresholds, and `Hero.svelte` is streamlined into a compact HUD header.

**Tech Stack:** Svelte 5, TypeScript, Vite, Vitest, Tauri v2, CSS custom properties.

---

### Task 1: Multi-Palette Theme Engine (`theme.ts`)

**Files:**
- Modify: `quota-tray/ui/src/lib/theme.ts`
- Test: `quota-tray/ui/src/lib/theme.test.ts`

- [ ] **Step 1: Write the failing test for palette management**

In `quota-tray/ui/src/lib/theme.test.ts`, append tests for `readPalette`, `applyPalette`, `initPalette`, and `cyclePalette`:

```typescript
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

  it("defaults to system mode", () => {
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
```

- [ ] **Step 2: Run test to verify it fails**

Run: `npm test` in `quota-tray/ui`
Expected: FAIL with "readPalette is not exported from ./theme"

- [ ] **Step 3: Implement multi-palette functions in `theme.ts`**

Update `quota-tray/ui/src/lib/theme.ts`:

```typescript
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
```

- [ ] **Step 4: Run test to verify it passes**

Run: `npm test` in `quota-tray/ui`
Expected: PASS (all tests passing)

- [ ] **Step 5: Commit**

```bash
git add ui/src/lib/theme.ts ui/src/lib/theme.test.ts
git commit -m "feat: add multi-palette theming engine with persistence and cycle support"
```

---

### Task 2: Segment Meter Helper Functions (`format.ts`)

**Files:**
- Modify: `quota-tray/ui/src/lib/format.ts`
- Test: `quota-tray/ui/src/lib/format.test.ts`

- [ ] **Step 1: Write the failing tests for segment calculations**

In `quota-tray/ui/src/lib/format.test.ts`, add tests for `segmentedActiveCount` and `isWarningSegment`:

```typescript
import { describe, expect, it } from "vitest";
import {
  formatHeadlinePercent,
  meterWidthPercent,
  segmentedActiveCount,
  isWarningSegment,
} from "./format";

describe("format", () => {
  it("formats headline percentage cleanly", () => {
    expect(formatHeadlinePercent(null)).toBe("—");
    expect(formatHeadlinePercent(undefined)).toBe("—");
    expect(formatHeadlinePercent(75.4)).toBe("75%");
    expect(formatHeadlinePercent(0)).toBe("0%");
  });

  it("clamps meter width percent between 0 and 100", () => {
    expect(meterWidthPercent(null)).toBeNull();
    expect(meterWidthPercent(50)).toBe(50);
    expect(meterWidthPercent(-5)).toBe(0);
    expect(meterWidthPercent(105)).toBe(100);
  });

  it("calculates discrete active segment count for a 12-segment meter", () => {
    expect(segmentedActiveCount(null)).toBeNull();
    expect(segmentedActiveCount(undefined)).toBeNull();
    expect(segmentedActiveCount(0)).toBe(0);
    expect(segmentedActiveCount(25)).toBe(3);   // 25% of 12 = 3
    expect(segmentedActiveCount(50)).toBe(6);   // 50% of 12 = 6
    expect(segmentedActiveCount(75)).toBe(9);   // 75% of 12 = 9
    expect(segmentedActiveCount(100)).toBe(12); // 100% of 12 = 12
    expect(segmentedActiveCount(105)).toBe(12); // Clamped
  });

  it("identifies warning segments for usage >= 75% (segment index >= 9 of 12)", () => {
    // 12 segments: indices 0..11. Indices 9, 10, 11 represent the upper 25% (>= 75%)
    expect(isWarningSegment(0)).toBe(false);
    expect(isWarningSegment(8)).toBe(false);
    expect(isWarningSegment(9)).toBe(true);
    expect(isWarningSegment(11)).toBe(true);
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `npm test` in `quota-tray/ui`
Expected: FAIL with "segmentedActiveCount is not exported from ./format"

- [ ] **Step 3: Implement segment calculations in `format.ts`**

Update `quota-tray/ui/src/lib/format.ts`:

```typescript
/** Never treat null/undefined as a live 0%. */
export function formatHeadlinePercent(used: number | null | undefined): string {
  if (used === null || used === undefined || Number.isNaN(used)) {
    return "—";
  }
  return `${Math.round(used)}%`;
}

export function meterWidthPercent(used: number | null | undefined): number | null {
  if (used === null || used === undefined || Number.isNaN(used)) {
    return null;
  }
  return Math.min(100, Math.max(0, used));
}

/**
 * Maps a percentage (0..100) to discrete illuminated segments (0..totalSegments).
 * Returns null if percentage is unknown/null.
 */
export function segmentedActiveCount(
  used: number | null | undefined,
  totalSegments = 12
): number | null {
  if (used === null || used === undefined || Number.isNaN(used)) {
    return null;
  }
  const clamped = Math.min(100, Math.max(0, used));
  return Math.round((clamped / 100) * totalSegments);
}

/**
 * Returns true if a segment index is in the warning/critical zone (top 25% of gauge).
 */
export function isWarningSegment(
  segmentIndex: number,
  totalSegments = 12
): boolean {
  return segmentIndex >= Math.floor(totalSegments * 0.75);
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `npm test` in `quota-tray/ui`
Expected: PASS (all 4 test files pass)

- [ ] **Step 5: Commit**

```bash
git add ui/src/lib/format.ts ui/src/lib/format.test.ts
git commit -m "feat: add segmentedActiveCount and isWarningSegment helpers"
```

---

### Task 3: Palette Tokens & HUD Styling (`app.css`)

**Files:**
- Modify: `quota-tray/ui/src/app.css`

- [ ] **Step 1: Define CSS custom properties for all 4 palettes in `app.css`**

Update `quota-tray/ui/src/app.css` with palette variables and segmented meter styling:

```css
/* Tokens and Multi-Palette Themes */

:root {
  color-scheme: light;
  --font: -apple-system, BlinkMacSystemFont, "SF Pro Display", "SF Pro Text", system-ui, sans-serif;
  --font-ui: -apple-system, BlinkMacSystemFont, "SF Pro Text", system-ui, sans-serif;
  --font-mono: ui-monospace, "SF Mono", Menlo, Consolas, monospace;

  --ease: cubic-bezier(0.22, 1, 0.36, 1);
  --ease-fill: cubic-bezier(0.16, 1, 0.3, 1);
  --fill-duration: 480ms;

  --radius: 12px;
  --radius-sm: 8px;

  /* Default Studio (Warm Terracotta & Craft Natural) */
  --sheet: rgba(255, 252, 248, 0.96);
  --popover-stroke: rgba(0, 0, 0, 0.12);
  --card-bg: rgba(245, 237, 230, 0.65);
  --card-border: #dfd2c4;
  --card-hover: rgba(245, 237, 230, 0.95);
  --ink: #2c2c2c;
  --ink-muted: #6b5d50;
  --chip-claude: #c96442;
  --chip-cursor: #5a7a6a;
  --chip-copilot: #3d5a80;
  --seg-active-claude: #c96442;
  --seg-active-cursor: #5a7a6a;
  --seg-active-copilot: #3d5a80;
  --seg-warn: #d94b34;
  --seg-inactive: #ebe0d4;
  --seg-glow-claude: rgba(201, 100, 66, 0.3);
  --seg-glow-cursor: rgba(90, 122, 106, 0.3);
  --seg-glow-copilot: rgba(61, 90, 128, 0.3);
  --seg-glow-warn: rgba(217, 75, 52, 0.45);
  --danger: #a33b2a;
  --ok: #5a7a6a;
  --pale: #fbf7f2;
  --cream: #f5ede6;
  --cream-deep: #ebe0d4;
  --cream-line: #dfd2c4;
}

/* Dark Mode base adjustments */
@media (prefers-color-scheme: dark) {
  :root:not([data-theme="light"]) {
    color-scheme: dark;
    --sheet: rgba(38, 30, 25, 0.96);
    --popover-stroke: rgba(255, 255, 255, 0.12);
    --card-bg: rgba(50, 40, 34, 0.7);
    --card-border: #4a3d34;
    --card-hover: rgba(50, 40, 34, 0.95);
    --ink: #f5ede6;
    --ink-muted: #b8a99a;
    --seg-inactive: #3a2f28;
    --pale: #231c18;
    --cream: #2a211c;
    --cream-deep: #3a2f28;
    --cream-line: #4a3d34;
  }
}

:root[data-theme="dark"] {
  color-scheme: dark;
  --sheet: rgba(38, 30, 25, 0.96);
  --popover-stroke: rgba(255, 255, 255, 0.12);
  --card-bg: rgba(50, 40, 34, 0.7);
  --card-border: #4a3d34;
  --card-hover: rgba(50, 40, 34, 0.95);
  --ink: #f5ede6;
  --ink-muted: #b8a99a;
  --seg-inactive: #3a2f28;
  --pale: #231c18;
  --cream: #2a211c;
  --cream-deep: #3a2f28;
  --cream-line: #4a3d34;
}

/* Palette 1: Studio Terracotta */
:root[data-palette="studio"] {
  --chip-claude: #c96442;
  --chip-cursor: #5a7a6a;
  --chip-copilot: #3d5a80;
  --seg-active-claude: #c96442;
  --seg-active-cursor: #5a7a6a;
  --seg-active-copilot: #3d5a80;
  --seg-warn: #d94b34;
  --seg-glow-claude: rgba(201, 100, 66, 0.35);
  --seg-glow-cursor: rgba(90, 122, 106, 0.35);
  --seg-glow-copilot: rgba(61, 90, 128, 0.35);
  --seg-glow-warn: rgba(217, 75, 52, 0.45);
}

/* Palette 2: Cyberpunk Neon Terminal */
:root[data-palette="cyberpunk"] {
  --chip-claude: #ff7a45;
  --chip-cursor: #00f5d4;
  --chip-copilot: #9d4edd;
  --seg-active-claude: #ff7a45;
  --seg-active-cursor: #00f5d4;
  --seg-active-copilot: #9d4edd;
  --seg-warn: #ff0055;
  --seg-glow-claude: 0 0 8px rgba(255, 122, 69, 0.6);
  --seg-glow-cursor: 0 0 8px rgba(0, 245, 212, 0.6);
  --seg-glow-copilot: 0 0 8px rgba(157, 78, 221, 0.6);
  --seg-glow-warn: 0 0 10px rgba(255, 0, 85, 0.75);
}

/* Palette 3: Tokyo Night */
:root[data-palette="tokyonight"] {
  --chip-claude: #ff9e64;
  --chip-cursor: #7aa2f7;
  --chip-copilot: #bb9af7;
  --seg-active-claude: #ff9e64;
  --seg-active-cursor: #7aa2f7;
  --seg-active-copilot: #bb9af7;
  --seg-warn: #f7768e;
  --seg-glow-claude: 0 0 7px rgba(255, 158, 100, 0.5);
  --seg-glow-cursor: 0 0 7px rgba(122, 162, 247, 0.5);
  --seg-glow-copilot: 0 0 7px rgba(187, 154, 247, 0.5);
  --seg-glow-warn: 0 0 9px rgba(247, 118, 142, 0.65);
}

/* Palette 4: Matrix Emerald */
:root[data-palette="matrix"] {
  --chip-claude: #66bb6a;
  --chip-cursor: #00e676;
  --chip-copilot: #4caf50;
  --seg-active-claude: #66bb6a;
  --seg-active-cursor: #00e676;
  --seg-active-copilot: #4caf50;
  --seg-warn: #ffd600;
  --seg-glow-claude: 0 0 7px rgba(102, 187, 106, 0.5);
  --seg-glow-cursor: 0 0 7px rgba(0, 230, 118, 0.6);
  --seg-glow-copilot: 0 0 7px rgba(76, 175, 80, 0.5);
  --seg-glow-warn: 0 0 9px rgba(255, 214, 0, 0.65);
}

* {
  box-sizing: border-box;
}

html,
body,
#app {
  margin: 0;
  min-height: 100%;
  height: 100%;
  background: transparent;
  color: var(--ink);
  font-family: var(--font-ui);
  -webkit-font-smoothing: antialiased;
}

@keyframes rise-in {
  from {
    opacity: 0;
    transform: translateY(6px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@keyframes pulse-heartbeat {
  0%, 100% {
    transform: scale(1);
    opacity: 0.8;
  }
  50% {
    transform: scale(1.15);
    opacity: 1;
  }
}

@keyframes seg-scan {
  0% { opacity: 0.3; }
  50% { opacity: 0.8; }
  100% { opacity: 0.3; }
}

@media (prefers-reduced-motion: reduce) {
  *,
  *::before,
  *::after {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
  }
}
```

- [ ] **Step 2: Verify CSS builds cleanly**

Run: `npm run build` in `quota-tray/ui`
Expected: PASS with 0 errors

- [ ] **Step 3: Commit**

```bash
git add ui/src/app.css
git commit -m "style: define multi-palette color variables and HUD animations"
```

---

### Task 4: ProviderRow Precision 12-Segment Meter (`ProviderRow.svelte`)

**Files:**
- Modify: `quota-tray/ui/src/lib/components/ProviderRow.svelte`

- [ ] **Step 1: Update `ProviderRow.svelte` with 12-segment meter and HUD card layout**

Rewrite `quota-tray/ui/src/lib/components/ProviderRow.svelte`:

```svelte
<script lang="ts">
  import { onMount } from "svelte";
  import type { ProviderSnapshot, UsageWindow } from "../types";
  import {
    formatHeadlinePercent,
    segmentedActiveCount,
    isWarningSegment,
  } from "../format";

  interface Props {
    snap: ProviderSnapshot;
    index?: number;
  }
  let { snap, index = 0 }: Props = $props();
  let fillReady = $state(false);

  onMount(() => {
    const id = requestAnimationFrame(() => {
      fillReady = true;
    });
    return () => cancelAnimationFrame(id);
  });

  const name = $derived(
    snap.provider === "Claude"
      ? "Claude"
      : snap.provider === "Cursor"
        ? "Cursor"
        : snap.provider === "Copilot"
          ? "Copilot"
          : "OpenAI"
  );
  const vendor = $derived(
    snap.provider === "Claude"
      ? "Anthropic"
      : snap.provider === "Cursor"
        ? "Anysphere"
        : snap.provider === "Copilot"
          ? "GitHub"
          : "OpenAI"
  );
  const providerClass = $derived(
    snap.provider === "Claude"
      ? "claude"
      : snap.provider === "Cursor"
        ? "cursor"
        : "copilot"
  );

  const totalSegments = 12;
  const activeSegments = $derived(
    segmentedActiveCount(snap.headline_percent, totalSegments)
  );
  const isHighUsage = $derived((snap.headline_percent ?? 0) >= 75);

  const remaining = $derived(
    snap.headline_percent == null
      ? null
      : Math.max(0, Math.round(100 - snap.headline_percent))
  );

  function windowUsed(w: UsageWindow): number | null {
    return "Percent" in w.kind ? w.kind.Percent.used : null;
  }

  function windowTokenDetails(w: UsageWindow): string | null {
    if ("TokenCount" in w.kind) {
      const u = w.kind.TokenCount.used;
      const l = w.kind.TokenCount.limit;
      return l != null ? `${u.toLocaleString()} / ${l.toLocaleString()}` : `${u.toLocaleString()} tokens`;
    }
    return null;
  }
</script>

<article class="provider-card" style={`animation-delay: ${60 + index * 50}ms`}>
  <div class="card-head">
    <div class="identity">
      <span class={`vendor-tag tag-${providerClass}`}>{vendor}</span>
      <span class="name">{name}</span>
      {#if snap.stale}
        <span class="stale-badge">Stale</span>
      {/if}
    </div>
    <div class="figures">
      <span class={`pct pct-${providerClass}`} class:warn={isHighUsage}>
        {formatHeadlinePercent(snap.headline_percent)}
      </span>
      {#if remaining != null}
        <span class="left">{remaining}% left</span>
      {:else}
        <span class="left">Waiting</span>
      {/if}
    </div>
  </div>

  <!-- 12-Segment Precision Bar -->
  <div class="segmented-track" class:stale={snap.stale} aria-label={`${name} usage: ${formatHeadlinePercent(snap.headline_percent)}`}>
    {#each Array(totalSegments) as _, i}
      {@const isActive = activeSegments != null && fillReady && i < activeSegments}
      {@const isWarn = isActive && isWarningSegment(i, totalSegments)}
      <div
        class="seg-block"
        class:active={isActive}
        class:warn={isWarn}
        class:inactive={!isActive && activeSegments != null}
        class:scanning={activeSegments == null}
        class:claude={isActive && !isWarn && snap.provider === "Claude"}
        class:cursor={isActive && !isWarn && snap.provider === "Cursor"}
        class:copilot={isActive && !isWarn && snap.provider !== "Claude" && snap.provider !== "Cursor"}
        style={`transition-delay: ${i * 24}ms; animation-delay: ${i * 60}ms`}
      ></div>
    {/each}
  </div>

  {#if snap.error}
    <p class="err">{snap.error}</p>
  {/if}

  {#if snap.windows.length > 0}
    <ul class="subwindows">
      {#each snap.windows as w (w.id)}
        <li class="subwindow-item">
          <span class="w-label">
            {w.label}
            {#if w.resets_at}
              <span class="w-reset">· Resets {w.resets_at}</span>
            {/if}
          </span>
          {#if windowTokenDetails(w)}
            <span class="w-val">{windowTokenDetails(w)}</span>
          {:else}
            <span class="w-val">{formatHeadlinePercent(windowUsed(w))}</span>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
</article>

<style>
  .provider-card {
    background: var(--card-bg);
    border: 1px solid var(--card-border);
    border-radius: var(--radius-sm);
    padding: 0.75rem 0.85rem;
    margin-bottom: 0.6rem;
    animation: rise-in 380ms var(--ease) both;
    transition: background 180ms ease, border-color 180ms ease;
  }
  .provider-card:hover {
    background: var(--card-hover);
  }
  .provider-card:last-child {
    margin-bottom: 0;
  }

  .card-head {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    gap: 0.5rem;
  }

  .identity {
    display: flex;
    align-items: center;
    gap: 0.45rem;
  }

  .vendor-tag {
    font-size: 0.62rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    padding: 1px 5px;
    border-radius: 4px;
    color: #ffffff;
  }
  .tag-claude {
    background: var(--chip-claude);
  }
  .tag-cursor {
    background: var(--chip-cursor);
  }
  .tag-copilot {
    background: var(--chip-copilot);
  }

  .name {
    font-weight: 650;
    font-size: 0.88rem;
    letter-spacing: -0.01em;
  }

  .stale-badge {
    font-size: 0.6rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--danger);
    background: color-mix(in srgb, var(--danger) 14%, transparent);
    border: 1px solid color-mix(in srgb, var(--danger) 30%, transparent);
    border-radius: 3px;
    padding: 0.05rem 0.3rem;
  }

  .figures {
    display: flex;
    align-items: baseline;
    gap: 0.35rem;
    font-family: var(--font-mono);
  }

  .pct {
    font-size: 0.95rem;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
    color: var(--ink);
  }
  .pct-claude {
    color: var(--chip-claude);
  }
  .pct-cursor {
    color: var(--chip-cursor);
  }
  .pct-copilot {
    color: var(--chip-copilot);
  }
  .pct.warn {
    color: var(--seg-warn);
  }

  .left {
    font-size: 0.68rem;
    color: var(--ink-muted);
  }

  /* 12-segment precision track */
  .segmented-track {
    display: flex;
    gap: 3px;
    height: 8px;
    margin-top: 0.5rem;
  }
  .segmented-track.stale {
    opacity: 0.45;
  }

  .seg-block {
    flex: 1;
    border-radius: 2px;
    background: var(--seg-inactive);
    transition: background 300ms var(--ease), box-shadow 300ms var(--ease);
  }

  .seg-block.claude {
    background: var(--seg-active-claude);
    box-shadow: var(--seg-glow-claude);
  }
  .seg-block.cursor {
    background: var(--seg-active-cursor);
    box-shadow: var(--seg-glow-cursor);
  }
  .seg-block.copilot {
    background: var(--seg-active-copilot);
    box-shadow: var(--seg-glow-copilot);
  }
  .seg-block.warn {
    background: var(--seg-warn);
    box-shadow: var(--seg-glow-warn);
  }

  .seg-block.scanning {
    animation: seg-scan 1.2s ease-in-out infinite;
  }

  .err {
    margin: 0.4rem 0 0;
    color: var(--danger);
    font-size: 0.72rem;
    line-height: 1.3;
  }

  /* Subwindows */
  .subwindows {
    list-style: none;
    margin: 0.5rem 0 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
  }

  .subwindow-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 0.7rem;
    padding: 0.25rem 0.45rem;
    border-radius: 5px;
    background: color-mix(in srgb, var(--pale) 60%, transparent);
  }

  .w-label {
    color: var(--ink-muted);
  }
  .w-reset {
    opacity: 0.75;
  }

  .w-val {
    font-weight: 650;
    font-family: var(--font-mono);
    color: var(--ink);
  }
</style>
```

- [ ] **Step 2: Build verification**

Run: `npm run build` in `quota-tray/ui`
Expected: PASS with 0 errors

- [ ] **Step 3: Commit**

```bash
git add ui/src/lib/components/ProviderRow.svelte
git commit -m "feat: implement 12-segment LED fill meter and HUD card in ProviderRow"
```

---

### Task 5: Compact HUD Header (`Hero.svelte`)

**Files:**
- Modify: `quota-tray/ui/src/lib/components/Hero.svelte`

- [ ] **Step 1: Modernize `Hero.svelte` into a streamlined HUD banner**

Rewrite `quota-tray/ui/src/lib/components/Hero.svelte`:

```svelte
<script lang="ts">
  import { formatHeadlinePercent } from "../format";
  import type { ProviderSnapshot } from "../types";

  interface Props {
    fill: number | null;
    staleGlobal: boolean;
    providers?: ProviderSnapshot[];
  }
  let { fill, staleGlobal, providers = [] }: Props = $props();

  const activeCount = $derived(
    providers.filter((p) => p.headline_percent != null).length
  );
  const hasErrors = $derived(providers.some((p) => p.error != null));

  const pulseTone = $derived(
    hasErrors
      ? "error"
      : staleGlobal
        ? "warn"
        : fill == null
          ? "wait"
          : "ok"
  );
  const peakText = $derived(
    fill != null ? `PEAK ${formatHeadlinePercent(fill)}` : "WAITING"
  );
</script>

<header class="hud-topbar">
  <div class="brand-group">
    <span class={`status-dot dot-${pulseTone}`} aria-label={`Status: ${pulseTone}`}></span>
    <h1 class="hud-title">Quota Tray</h1>
    <span class="active-badge">{activeCount} ACTIVE</span>
  </div>

  <div class="peak-badge" class:warn={fill != null && fill >= 75}>
    {peakText}
  </div>
</header>

<style>
  .hud-topbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem 1rem 0.65rem;
    border-bottom: 0.5px solid var(--popover-stroke);
    animation: rise-in 320ms var(--ease) both;
  }

  .brand-group {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
    box-shadow: 0 0 0 2px rgba(0, 0, 0, 0.08);
  }
  .dot-ok {
    background: var(--ok);
    animation: pulse-heartbeat 2.4s ease-in-out infinite;
  }
  .dot-warn {
    background: var(--danger);
  }
  .dot-error {
    background: #e07060;
    animation: pulse-heartbeat 1.2s ease-in-out infinite;
  }
  .dot-wait {
    background: var(--ink-muted);
  }

  .hud-title {
    margin: 0;
    font-size: 0.92rem;
    font-weight: 700;
    letter-spacing: -0.02em;
    color: var(--ink);
  }

  .active-badge {
    font-size: 0.62rem;
    font-family: var(--font-mono);
    font-weight: 650;
    letter-spacing: 0.04em;
    color: var(--ink-muted);
    background: color-mix(in srgb, var(--card-bg) 80%, transparent);
    border: 0.5px solid var(--card-border);
    padding: 1px 5px;
    border-radius: 4px;
  }

  .peak-badge {
    font-family: var(--font-mono);
    font-size: 0.72rem;
    font-weight: 750;
    padding: 2px 7px;
    border-radius: 5px;
    color: var(--ink);
    background: color-mix(in srgb, var(--pale) 85%, transparent);
    border: 0.5px solid var(--card-border);
  }
  .peak-badge.warn {
    color: var(--seg-warn);
    background: color-mix(in srgb, var(--seg-warn) 12%, transparent);
    border-color: color-mix(in srgb, var(--seg-warn) 35%, transparent);
  }
</style>
```

- [ ] **Step 2: Build verification**

Run: `npm run build` in `quota-tray/ui`
Expected: PASS with 0 errors

- [ ] **Step 3: Commit**

```bash
git add ui/src/lib/components/Hero.svelte
git commit -m "feat: modernize Hero into compact HUD topbar with status dot and peak badge"
```

---

### Task 6: Main Popover Tray Updates & Quick Palette Switcher (`App.svelte`)

**Files:**
- Modify: `quota-tray/ui/src/App.svelte`

- [ ] **Step 1: Wire up palette initialization, storage sync, and footer switcher**

Update `quota-tray/ui/src/App.svelte`:

```svelte
<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import Hero from "./lib/components/Hero.svelte";
  import ProviderRow from "./lib/components/ProviderRow.svelte";
  import EmptyState from "./lib/components/EmptyState.svelte";
  import { fetchTrayState, onTrayStateUpdated } from "./lib/api";
  import { initPalette, applyPalette, cyclePalette, readPalette, type PaletteId } from "./lib/theme";
  import type { TrayState } from "./lib/types";

  let state = $state<TrayState | null>(null);
  let loadError = $state<string | null>(null);
  let currentPalette = $state<PaletteId>("studio");

  async function refresh() {
    try {
      state = await fetchTrayState();
      loadError = null;
    } catch (e) {
      loadError = e instanceof Error ? e.message : String(e);
    }
  }

  function handleCyclePalette() {
    const next = cyclePalette(currentPalette);
    currentPalette = next;
    applyPalette(next);
  }

  onMount(() => {
    currentPalette = initPalette();
    refresh();

    let unlisten: (() => void) | undefined;
    onTrayStateUpdated(() => {
      refresh();
    }).then((u) => {
      unlisten = u;
    });

    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        void getCurrentWindow().hide();
      }
    };
    window.addEventListener("keydown", onKey);

    const onStorage = (e: StorageEvent) => {
      if (e.key === "quota-tray-palette") {
        currentPalette = readPalette();
        applyPalette(currentPalette);
      }
    };
    window.addEventListener("storage", onStorage);

    return () => {
      unlisten?.();
      window.removeEventListener("keydown", onKey);
      window.removeEventListener("storage", onStorage);
    };
  });

  const hasAnyReading = $derived(
    !!state &&
      state.providers.some(
        (p) => p.headline_percent != null || p.windows.length > 0
      )
  );
  const staleGlobal = $derived(
    !!state && hasAnyReading && state.providers.every((p) => p.stale)
  );
</script>

<main class="panel">
  <Hero
    fill={state?.shared_mascot_fill ?? null}
    staleGlobal={staleGlobal}
    providers={state?.providers ?? []}
  />

  {#if loadError}
    <p class="banner warn">{loadError}</p>
  {/if}

  {#if state && state.providers.length === 0}
    <EmptyState
      title="No providers yet"
      body="Turn on Claude, Cursor, or Copilot in Settings. Tokens stay on this Mac."
    />
  {:else if state && state.providers.length > 0}
    <section class="providers-section" aria-label="Providers">
      <div class="list">
        {#each state.providers as snap, i (snap.provider)}
          <ProviderRow {snap} index={i} />
        {/each}
      </div>
    </section>
  {/if}

  <footer class="hud-footer">
    <div class="footer-left">
      <a class="btn-action" href="settings.html">Settings</a>
      <button type="button" class="btn-action" onclick={handleCyclePalette} title="Cycle theme palette">
        🎨 {currentPalette}
      </button>
    </div>
    <div class="footer-right">
      <button type="button" class="btn-action" onclick={refresh}>↻ Refresh</button>
    </div>
  </footer>
</main>

<style>
  .panel {
    min-height: 100vh;
    display: flex;
    flex-direction: column;
    background: var(--sheet);
    border: 0.5px solid var(--popover-stroke);
    border-radius: 12px;
    overflow: hidden;
    margin: 0;
  }

  .providers-section {
    padding: 0.75rem 0.85rem 0.5rem;
    animation: rise-in 360ms var(--ease) both;
  }

  .list {
    display: flex;
    flex-direction: column;
  }

  .hud-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.65rem 0.85rem;
    margin-top: auto;
    border-top: 0.5px solid var(--popover-stroke);
    background: color-mix(in srgb, var(--pale) 40%, transparent);
  }

  .footer-left,
  .footer-right {
    display: flex;
    align-items: center;
    gap: 0.45rem;
  }

  .btn-action {
    appearance: none;
    background: transparent;
    border: 0.5px solid var(--popover-stroke);
    border-radius: 5px;
    color: var(--ink);
    padding: 0.28rem 0.55rem;
    font: inherit;
    font-size: 0.75rem;
    font-weight: 550;
    text-decoration: none;
    cursor: pointer;
    text-transform: capitalize;
    transition: background 140ms ease;
  }
  .btn-action:hover {
    background: color-mix(in srgb, var(--card-bg) 90%, transparent);
  }

  .banner {
    margin: 0.6rem 0.85rem 0.3rem;
    padding: 0.45rem 0.65rem;
    color: var(--ink-muted);
    font-size: 0.82rem;
    line-height: 1.3;
    background: var(--pale);
    border: 0.5px solid var(--popover-stroke);
    border-radius: 6px;
  }
  .banner.warn {
    color: var(--danger);
  }
</style>
```

- [ ] **Step 2: Build verification**

Run: `npm run build` in `quota-tray/ui`
Expected: PASS with 0 errors

- [ ] **Step 3: Commit**

```bash
git add ui/src/App.svelte
git commit -m "feat: integrate palette synchronization and quick switcher in tray footer"
```

---

### Task 7: Settings App Palette Selector (`SettingsApp.svelte`)

**Files:**
- Modify: `quota-tray/ui/src/SettingsApp.svelte`

- [ ] **Step 1: Add Color Palette radio cards in `SettingsApp.svelte`**

Update `quota-tray/ui/src/SettingsApp.svelte` to import `PALETTE_OPTIONS`, `initPalette`, `applyPalette`, `readPalette`, and `type PaletteId`, and add a dedicated "Color Palette" card under Appearance:

```svelte
<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { clampPollInterval } from "./lib/settings_validate";
  import {
    applyThemeMode,
    initTheme,
    readPalette,
    applyPalette,
    initPalette,
    PALETTE_OPTIONS,
    type ThemeMode,
    type PaletteId,
  } from "./lib/theme";

  type ProviderId = "Claude" | "Cursor" | "Copilot" | "OpenAI";
  type ClaudeHeadline = "FiveHour" | "SevenDay" | "Highest";

  interface AppConfig {
    poll_interval_secs: number;
    enabled: ProviderId[];
    claude_headline: ClaudeHeadline;
  }

  let config = $state<AppConfig>({
    poll_interval_secs: 60,
    enabled: ["Claude"],
    claude_headline: "Highest",
  });
  let theme = $state<ThemeMode>("system");
  let palette = $state<PaletteId>("studio");
  let saved = $state(false);
  let error = $state<string | null>(null);

  const allProviders: { id: ProviderId; blurb: string }[] = [
    { id: "Claude", blurb: "Keychain / ~/.claude credentials → Anthropic usage" },
    { id: "Cursor", blurb: "Local state.vscdb token → Cursor period usage" },
    { id: "Copilot", blurb: "apps.json / hosts.json → GitHub Copilot user API" },
  ];

  const themeOptions: { id: ThemeMode; label: string; hint: string }[] = [
    { id: "system", label: "System", hint: "Match macOS" },
    { id: "light", label: "Light", hint: "Cream" },
    { id: "dark", label: "Dark", hint: "Charcoal" },
  ];

  onMount(async () => {
    theme = initTheme();
    palette = initPalette();
    try {
      config = await invoke<AppConfig>("get_config");
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  });

  function setTheme(mode: ThemeMode) {
    theme = mode;
    applyThemeMode(mode);
  }

  function setPalette(id: PaletteId) {
    palette = id;
    applyPalette(id);
  }

  function toggle(id: ProviderId, on: boolean) {
    if (on && !config.enabled.includes(id)) {
      config = { ...config, enabled: [...config.enabled, id] };
    } else if (!on) {
      config = { ...config, enabled: config.enabled.filter((x) => x !== id) };
    }
    saved = false;
  }

  async function save() {
    try {
      const next = {
        ...config,
        poll_interval_secs: clampPollInterval(config.poll_interval_secs),
      };
      config = await invoke<AppConfig>("set_config", { config: next });
      saved = true;
      error = null;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  function onPollInput(e: Event) {
    const v = Number((e.currentTarget as HTMLInputElement).value);
    config = { ...config, poll_interval_secs: v };
    saved = false;
  }
</script>

<main class="settings">
  <header class="head">
    <a class="back" href="index.html" aria-label="Back to panel">←</a>
    <div>
      <h1>Settings</h1>
      <p class="blurb">Credentials stay on this Mac. We only call vendor APIs.</p>
    </div>
  </header>

  <section class="card">
    <h2>Appearance Mode</h2>
    <div class="theme-grid" role="radiogroup" aria-label="Theme">
      {#each themeOptions as opt}
        <button
          type="button"
          class="theme-card"
          class:active={theme === opt.id}
          role="radio"
          aria-checked={theme === opt.id}
          onclick={() => setTheme(opt.id)}
        >
          <span class={`swatch swatch-${opt.id}`}></span>
          <span class="theme-label">{opt.label}</span>
          <span class="theme-hint">{opt.hint}</span>
        </button>
      {/each}
    </div>
  </section>

  <section class="card">
    <h2>Color Palette</h2>
    <div class="palette-grid" role="radiogroup" aria-label="Color Palette">
      {#each PALETTE_OPTIONS as p}
        <button
          type="button"
          class="palette-card"
          class:active={palette === p.id}
          role="radio"
          aria-checked={palette === p.id}
          onclick={() => setPalette(p.id)}
        >
          <div class="palette-header">
            <span class="palette-title">{p.label}</span>
            <div class="swatches-row">
              {#each p.swatches as s}
                <span class="swatch-dot" style={`background: ${s}`}></span>
              {/each}
            </div>
          </div>
          <p class="palette-desc">{p.description}</p>
        </button>
      {/each}
    </div>
  </section>

  <section class="card">
    <h2>Providers</h2>
    {#each allProviders as p}
      <label class="toggle-row">
        <div class="copy">
          <span class="title">{p.id}</span>
          <span class="desc">{p.blurb}</span>
        </div>
        <input
          class="switch"
          type="checkbox"
          checked={config.enabled.includes(p.id)}
          onchange={(e) => toggle(p.id, e.currentTarget.checked)}
        />
      </label>
    {/each}
  </section>

  <section class="card">
    <div class="card-top">
      <h2>Poll interval</h2>
      <span class="value">{config.poll_interval_secs}s</span>
    </div>
    <input
      class="range"
      type="range"
      min="60"
      max="1800"
      step="60"
      value={config.poll_interval_secs}
      oninput={onPollInput}
    />
  </section>

  <div class="actions">
    <button type="button" class="btn primary" onclick={save}>Save</button>
    {#if saved}
      <span class="saved">Saved</span>
    {/if}
    {#if error}
      <span class="err">{error}</span>
    {/if}
  </div>
</main>

<style>
  .settings {
    max-width: 480px;
    margin: 0 auto;
    padding: 1rem 1.25rem 2rem;
    font-family: var(--font-ui);
    color: var(--ink);
    background: var(--sheet);
  }

  .head {
    display: flex;
    align-items: center;
    gap: 0.85rem;
    margin-bottom: 1.2rem;
  }

  .back {
    display: inline-grid;
    place-items: center;
    width: 32px;
    height: 32px;
    border-radius: 50%;
    border: 0.5px solid var(--popover-stroke);
    color: var(--ink);
    text-decoration: none;
    font-size: 1.1rem;
    background: var(--card-bg);
  }

  h1 {
    margin: 0;
    font-size: 1.25rem;
    font-weight: 700;
  }

  .blurb {
    margin: 0.2rem 0 0;
    font-size: 0.76rem;
    color: var(--ink-muted);
  }

  .card {
    background: var(--card-bg);
    border: 0.5px solid var(--card-border);
    border-radius: var(--radius);
    padding: 0.9rem 1rem;
    margin-bottom: 1rem;
  }

  .card h2 {
    margin: 0 0 0.75rem;
    font-size: 0.85rem;
    font-weight: 700;
    letter-spacing: -0.01em;
  }

  .theme-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 0.6rem;
  }

  .theme-card {
    appearance: none;
    border: 1px solid var(--card-border);
    border-radius: 8px;
    background: var(--pale);
    padding: 0.65rem 0.5rem;
    text-align: center;
    cursor: pointer;
    color: var(--ink);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.35rem;
  }
  .theme-card.active {
    border-color: var(--chip-claude);
    box-shadow: 0 0 0 1.5px var(--chip-claude);
  }

  .swatch {
    width: 22px;
    height: 22px;
    border-radius: 50%;
    border: 1px solid var(--popover-stroke);
  }
  .swatch-system {
    background: linear-gradient(135deg, #ffffff 50%, #2c2c2c 50%);
  }
  .swatch-light {
    background: #fbf7f2;
  }
  .swatch-dark {
    background: #231c18;
  }

  .theme-label {
    font-size: 0.78rem;
    font-weight: 600;
  }
  .theme-hint {
    font-size: 0.68rem;
    color: var(--ink-muted);
  }

  /* Palette Grid */
  .palette-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.6rem;
  }

  .palette-card {
    appearance: none;
    border: 1px solid var(--card-border);
    border-radius: 8px;
    background: var(--pale);
    padding: 0.65rem 0.75rem;
    text-align: left;
    cursor: pointer;
    color: var(--ink);
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    transition: border-color 150ms ease;
  }
  .palette-card.active {
    border-color: var(--chip-claude);
    box-shadow: 0 0 0 1.5px var(--chip-claude);
  }

  .palette-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .palette-title {
    font-size: 0.8rem;
    font-weight: 650;
  }

  .swatches-row {
    display: flex;
    gap: 4px;
  }

  .swatch-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.2);
  }

  .palette-desc {
    margin: 0;
    font-size: 0.68rem;
    color: var(--ink-muted);
    line-height: 1.3;
  }

  /* Providers */
  .toggle-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.55rem 0;
    border-top: 0.5px solid var(--popover-stroke);
    cursor: pointer;
  }
  .toggle-row:first-of-type {
    border-top: none;
    padding-top: 0;
  }

  .copy {
    display: flex;
    flex-direction: column;
  }
  .title {
    font-weight: 600;
    font-size: 0.86rem;
  }
  .desc {
    font-size: 0.72rem;
    color: var(--ink-muted);
  }

  .switch {
    width: 40px;
    height: 22px;
    cursor: pointer;
  }

  .card-top {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
  }
  .value {
    font-size: 0.82rem;
    font-weight: 700;
    font-family: var(--font-mono);
  }

  .range {
    width: 100%;
    margin-top: 0.4rem;
  }

  .actions {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin-top: 1.25rem;
  }

  .btn.primary {
    appearance: none;
    background: var(--chip-claude);
    color: white;
    border: none;
    border-radius: 6px;
    padding: 0.45rem 1.2rem;
    font-weight: 650;
    font-size: 0.85rem;
    cursor: pointer;
  }

  .saved {
    font-size: 0.82rem;
    font-weight: 600;
    color: var(--ok);
  }
  .err {
    font-size: 0.82rem;
    color: var(--danger);
  }
</style>
```

- [ ] **Step 2: Build verification**

Run: `npm run build` in `quota-tray/ui`
Expected: PASS with 0 errors

- [ ] **Step 3: Commit**

```bash
git add ui/src/SettingsApp.svelte
git commit -m "feat: add Color Palette selection grid in SettingsApp"
```

---

### Task 8: Test Suite & End-to-End Build Verification

**Files:**
- Test: All tests in `quota-tray/ui`

- [ ] **Step 1: Run full unit test suite**

Run: `npm test` in `quota-tray/ui`
Expected: All tests pass across `theme.test.ts`, `format.test.ts`, and `settings_validate.test.ts`.

- [ ] **Step 2: Run production build**

Run: `npm run build` in `quota-tray/ui`
Expected: `dist/index.html` and `dist/settings.html` bundled successfully without errors or warnings.

- [ ] **Step 3: Final verification commit**

```bash
git add .
git commit -m "chore: verify full test suite and production build for Quota Tray HUD"
```
