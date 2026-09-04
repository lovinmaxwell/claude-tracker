# Quota Tray: Precision Segmented HUD & Multi-Palette Theming Design

## Overview
Transform the Quota Tray menu bar popover from a basic line meter into a high-density, engineer-grade **Precision Segmented HUD** with selectable color palettes. The redesign improves quota readability, modernizes the popover visual hierarchy, and introduces a multi-palette theming engine supporting 4 curated visual themes.

---

## 1. Visual Design: Precision Segmented HUD

### 1.1 The 12-Segment Precision Meter
Instead of a continuous hairline progress bar, each provider's headline quota is represented by a discrete 12-segment LED bar:
* **Active Segments Count:** `Math.round((percentage / 100) * 12)` (clamped between 0 and 12).
* **Segment Illumination:**
  * Active segments glow with provider-specific theme colors.
  * Inactive segments rest in a recessed, low-contrast neutral track block.
  * **Warning Threshold (>= 75%):** Segments above index 8 (representing usage $\ge 75\%$) switch to a high-visibility alert/warning accent (e.g. coral/amber/crimson depending on palette) with outer glow.
* **Entrance Animation:** Staggered micro-animations (`fill-x` or sequential brightness step) when opening the tray or switching providers.
* **Stale / Unknown States:**
  * **Stale:** Reduced opacity ($40\%$) on active segments with a subtle "STALE" tag and amber indicator.
  * **No Reading / Loading:** Animated pulsing scan line across inactive tracks.

### 1.2 Layout & Hierarchy
* **HUD Header (`Hero.svelte`):**
  * Replaces the oversized circular mascot ring with a compact, unified technical header.
  * Heartbeat status pulse dot: Green (all healthy), Amber (stale data), Red (provider error).
  * Brand title: "Quota Tray" with active provider count indicator (`3 ACTIVE`).
  * Peak badge: High-contrast `PEAK XX%` chip summarizing the highest quota used among active providers.
* **Provider Cards (`ProviderRow.svelte`):**
  * Grouped inset card structure with hairline borders and subtle translucency.
  * Monospace tabular headline percentage (e.g. `76.0%`) paired with `% left` indicator.
  * Secondary window sub-chips (e.g. 5h Session, Weekly) with reset countdowns (`Resets 2h 15m`) and formatted token counters (`175 / 500`).
* **HUD Footer (`App.svelte`):**
  * Quick Palette Switcher button (`🎨 Theme`) to cycle palettes directly from the tray.
  * Monospace "UPDATED JUST NOW" relative timestamp.
  * Direct links to `Settings` and manual `Refresh`.

---

## 2. Multi-Palette Theming Engine

The app expands its theming capabilities from binary Light/Dark into a dual-axis system:
1. **Appearance Mode:** `system` | `light` | `dark` (manages background canvas and contrast base).
2. **Color Palette:** `studio` | `cyberpunk` | `tokyonight` | `matrix` (manages meter accents, glows, and card highlights).

### 2.1 Supported Palettes

| Palette ID | Name | Claude Accent | Cursor Accent | Copilot Accent | Warning Accent | Mood |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `studio` | Warm Studio Terracotta | `#c96442` (Terracotta) | `#5a7a6a` (Sage) | `#3d5a80` (Denim) | `#d94b34` (Coral) | Artisanal, calm, low eye strain |
| `cyberpunk` | Neon Terminal | `#ff7a45` (Electric Amber) | `#00f5d4` (Laser Cyan) | `#9d4edd` (Ultraviolet) | `#ff0055` (Laser Crimson) | High-contrast dark OLED, glowing HUD |
| `tokyonight` | Tokyo Night | `#ff9e64` (Warm Peach) | `#7aa2f7` (Sky Blue) | `#bb9af7` (Lavender) | `#f7768e` (Synthwave Pink) | Twilight indigo, aesthetic cyberpunk |
| `matrix` | Matrix Emerald | `#66bb6a` (Phosphor Green) | `#00e676` (Mint Green) | `#4caf50` (Forest Green) | `#ffd600` (Signal Gold) | Classic hacker terminal, phosphor glow |

### 2.2 Storage & Window Synchronization
* **Storage Key:** `quota-tray-palette` in `localStorage`.
* **HTML Attribute:** Applied as `data-palette="<palette-id>"` on `document.documentElement`.
* **Cross-Window Sync:** Both `App.svelte` (popover) and `SettingsApp.svelte` (settings window) listen to standard `storage` events so changes in Settings update the tray popover instantaneously without app restarts.

---

## 3. Component & Module Architecture

### 3.1 `ui/src/lib/theme.ts`
* Add types:
  ```typescript
  export type PaletteId = "studio" | "cyberpunk" | "tokyonight" | "matrix";
  export interface PaletteOption {
    id: PaletteId;
    label: string;
    description: string;
    swatches: string[];
  }
  ```
* Functions:
  * `readPalette(): PaletteId`
  * `applyPalette(id: PaletteId): void`
  * `initPalette(): PaletteId`
  * `cyclePalette(current: PaletteId): PaletteId`
  * `PALETTE_OPTIONS: PaletteOption[]`

### 3.2 `ui/src/app.css`
* Define CSS variable tokens scoped by `[data-palette="..."]`:
  * `--seg-active-claude`, `--seg-active-cursor`, `--seg-active-copilot`
  * `--seg-warn`
  * `--seg-inactive`
  * `--seg-glow-claude`, `--seg-glow-cursor`, `--seg-glow-warn`
  * `--hud-card-bg`, `--hud-card-border`
* Responsive styles adapting each palette cleanly across both Light and Dark macOS modes.

### 3.3 `ui/src/lib/components/ProviderRow.svelte`
* Replace single continuous `.fill` bar with 12 `.seg-block` divs inside `.segmented-track`.
* Add computation for active segments and warning states based on `snap.headline_percent`.
* Format secondary windows with compact reset countdowns and token counts.

### 3.4 `ui/src/lib/components/Hero.svelte`
* Redesign into the compact HUD header with pulse status dot, app title, and peak quota badge.

### 3.5 `ui/src/App.svelte`
* Mount `initPalette()` and listen for `storage` updates.
* Add palette switcher button (🎨) in the footer.
* Update layout container to support HUD card styling.

### 3.6 `ui/src/SettingsApp.svelte`
* Add "Color Palette" card under Appearance settings with interactive radio buttons showing palette swatches and labels.

---

## 4. Error Handling & Edge Cases
* **Missing or Null Headline Percent:** Displays neutral scanning track with "WAITING" label.
* **Stale Providers:** Applies `$stale` styling (dimmed LED blocks + "STALE" badge) without breaking layout.
* **Provider Error:** Shows inline error callout with muted LED track.
* **Fallback Theme:** If `localStorage` has invalid or missing values, default gracefully to `studio` palette and `system` mode.

---

## 5. Verification & Testing

### 5.1 Unit Tests (`ui/src/lib/theme.test.ts`)
* Test `readPalette()` default fallback to `"studio"`.
* Test `applyPalette()` DOM attribute update and `localStorage` write.
* Test `cyclePalette()` rotating cleanly through all 4 palettes.

### 5.2 Build & Integration Tests
* `npm run check` in `quota-tray/ui` to verify Svelte 5 and TypeScript typings.
* `npm run build` in `quota-tray/ui` to verify asset generation and bundle health.
* Visual testing in Tauri window across all 4 palettes in light and dark modes.
