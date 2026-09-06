# Menubar Pill-Ring Tray Icon Design

## Overview
Redesign the macOS menubar tray icon for Quota Tray. Replace the continuous circular sweep ring with a **12-pill segmented radial ring** that fills pill-by-pill based on quota usage, matching the HUD's 12-segment progress bar aesthetic. The icon features the pixel mascot centered inside the ring, renders on a 100% transparent canvas with zero background box or border, and operates in pure icon mode (no text or badges in the menubar).

---

## 1. Visual Specification

### 1.1 Transparent Canvas
* The tray icon is rendered on a transparent RGBA canvas (`32x32` standard, `44x44` Retina `@2x`).
* Zero background box, border, or container fills. The icon sits directly and natively on top of the macOS menu bar and wallpaper.

### 1.2 Center Mascot
* The pixel mascot remains positioned in the center of the icon canvas.
* The body fill animates/renders from feet upward based on `shared_mascot_fill` (0% to 100%).

### 1.3 12-Pill Segmented Ring
* The perimeter around the mascot is partitioned into **12 discrete radial curved pill segments**.
* **Capsule Pill Geometry:** Each segment is rendered with rounded semicircular caps on both ends rather than flat-cut edges, creating distinct floating pill capsules.
* **Sequential Color Fill by Usage:**
  * Active pill count: `filled = ((usage_percentage / 100.0) * 12.0).round().clamp(0.0, 12.0) as usize`.
  * **Inactive Pills ($i \ge \text{filled}$):** Painted in a subtle neutral track color (`rgba(255, 255, 255, 0.22)` in dark mode, `rgba(0, 0, 0, 0.15)` in light mode, or `SEGMENT_MUTED`).
  * **Normal Active Pills ($i < \text{filled}$ and $i < 9$):** Filled with the active provider / palette color (e.g. Anthropic Terracotta `#C96442`, Cursor `#5A7A6A`, Copilot `#3D5A80`).
  * **Warning Threshold Pills ($i < \text{filled}$ and $i \ge 9$):** For usage $\ge 75\%$, the upper 3 pill segments illuminate in alert coral/red (`#D94B34`) with a soft glow to signal high quota consumption directly in the menubar.
* **Multi-Provider Distribution:**
  * When multiple providers are enabled, the 12 pills are partitioned into proportional sectors per provider (e.g. 6 pills for Claude, 6 pills for Cursor), with each sector filling its pills based on that provider's headline percentage.

### 1.4 Menubar Appearance
* Pure icon mode: No percentage badge or title string next to the icon in the macOS menu bar.
* Tooltip remains active on hover, displaying provider quota details (`Claude 76% · Cursor 35%`).

---

## 2. Technical Architecture

### 2.1 Drawing & Rasterization (`crates/core/src/icon.rs`)
* Add pill segment rasterizer:
  * For each of the 12 segment angles $\theta_i = -\frac{\pi}{2} + i \cdot \frac{2\pi}{12}$:
    * Arc span: $\Delta\theta \approx 20^\circ$, angular gap $\approx 10^\circ$.
    * Radial distance: $R_{\text{mid}} \approx 13.0$px, thickness $W \approx 3.0$px.
    * For each pixel $(x, y)$: calculate distance from the pixel center to the curved capsule pill spine. If within $W / 2$, calculate alpha/anti-aliasing and blend the color into the canvas.
* Expose `paint_tray_icon` and `render_tray_rgba` updated with the 12-pill capsule algorithm.

### 2.2 Tauri Menubar Setup (`src-tauri/src/tray.rs` & `poll_loop.rs`)
* Pre-render an initial waiting 12-pill icon on app launch during `setup_tray` so the menubar icon is immediately visible without waiting for the first network tick.
* `poll_loop.rs` updates the tray icon via `apply_tray_icon` on every poll interval and state change.

---

## 3. Verification & Testing Plan
* **Unit & Raster Tests (`crates/core/tests/` & `icon.rs`):**
  * Test that `render_tray_rgba` generates non-empty RGBA buffers.
  * Test that 0%, 25% (3 pills), 50% (6 pills), 75% (9 pills), and 100% (12 pills) produce correct pixel color transitions without crashing.
* **Cargo Check & Build:**
  * Verify `cargo test` in `crates/core`.
  * Verify `cargo check` and `cargo tauri build` in `quota-tray/src-tauri`.
* **Visual & System Verification:**
  * Install updated `Quota Tray.app` into `/Applications/`.
  * Launch and verify that the menubar displays the transparent mascot with 12 pill capsules filling with color.
