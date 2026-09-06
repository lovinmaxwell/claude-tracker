# Menubar Pill-Ring Tray Icon Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Redesign the macOS menubar tray icon to feature 12 discrete curved pill capsules arranged in a circular ring around the centered mascot, filling pill-by-pill with color based on quota usage (with warning coral at $\ge 75\%$), on a 100% transparent canvas without background boxes or percentage badges.

**Architecture:** Implement anti-aliased curved pill capsule rasterization with rounded semicircular caps in `quota-tray/crates/core/src/icon.rs`. Replace flat sweep arcs with discrete 12-pill sequential coloring. Pre-render initial tray icon at app launch in `quota-tray/src-tauri/src/tray.rs` for instant visibility.

**Tech Stack:** Rust, `image` crate, Tauri v2 `TrayIcon`, macOS system menubar.

---

### Task 1: Pill Capsule Radial Ring Rasterizer & Tests (`icon.rs`)

**Files:**
- Modify: `quota-tray/crates/core/src/icon.rs`
- Test: `quota-tray/crates/core/tests/icon_render.rs`

- [ ] **Step 1: Write the failing tests for 12-pill segment icon rendering**

In `quota-tray/crates/core/tests/icon_render.rs`, add tests verifying 12-pill segment behavior across quota levels:

```rust
#[test]
fn pill_ring_differs_across_quota_thresholds() {
    let empty_state = TrayState {
        providers: vec![snap(ProviderId::Claude, Some(0.0), false)],
        shared_mascot_fill: Some(0.0),
    };
    let quarter_state = TrayState {
        providers: vec![snap(ProviderId::Claude, Some(25.0), false)],
        shared_mascot_fill: Some(25.0),
    };
    let warn_state = TrayState {
        providers: vec![snap(ProviderId::Claude, Some(75.0), false)],
        shared_mascot_fill: Some(75.0),
    };
    let full_state = TrayState {
        providers: vec![snap(ProviderId::Claude, Some(100.0), false)],
        shared_mascot_fill: Some(100.0),
    };

    let empty_buf = render_tray_rgba(&empty_state, 32);
    let quarter_buf = render_tray_rgba(&quarter_state, 32);
    let warn_buf = render_tray_rgba(&warn_state, 32);
    let full_buf = render_tray_rgba(&full_state, 32);

    assert_ne!(empty_buf, quarter_buf);
    assert_ne!(quarter_buf, warn_buf);
    assert_ne!(warn_buf, full_buf);

    // Canvas edges (0, 0) must be 100% transparent (no background container)
    assert_eq!(empty_buf[3], 0, "corner pixel alpha must be 0 (transparent)");
    assert_eq!(quarter_buf[3], 0, "corner pixel alpha must be 0 (transparent)");
    assert_eq!(warn_buf[3], 0, "corner pixel alpha must be 0 (transparent)");
}

#[test]
fn warning_threshold_activates_alert_color_on_upper_pills() {
    let normal_70 = TrayState {
        providers: vec![snap(ProviderId::Claude, Some(70.0), false)],
        shared_mascot_fill: Some(70.0),
    };
    let alert_80 = TrayState {
        providers: vec![snap(ProviderId::Claude, Some(80.0), false)],
        shared_mascot_fill: Some(80.0),
    };

    let png_70 = paint_tray_icon(&normal_70);
    let png_80 = paint_tray_icon(&alert_80);

    assert_ne!(png_70, png_80);
}
```

- [ ] **Step 2: Run test to verify it compiles and runs**

Run: `cargo test -p quota_tray_core --test icon_render`
Expected: Output will pass or fail based on current implementation.

- [ ] **Step 3: Implement anti-aliased 12-pill capsule ring in `icon.rs`**

Update `quota-tray/crates/core/src/icon.rs` with `paint_pill_capsule` and `paint_12_pill_ring`:
1. Define `WARNING_CORAL: RgbaColor = RgbaColor { r: 0xD9, g: 0x4B, b: 0x34, a: 255 }`.
2. Define `TRACK_MUTED: RgbaColor = RgbaColor { r: 0xDF, g: 0xD2, b: 0xC4, a: 70 }`.
3. For each pixel $(x, y)$, calculate exact distance to the curved pill capsule centerline with semicircular rounded end caps.
4. Calculate active pill count: `active_pills = ((pct / 100.0) * 12.0).round().clamp(0.0, 12.0) as usize`.
5. For pills $0..\text{active\_pills}$, fill with brand color, or `WARNING_CORAL` if pill index $\ge 9$ (representing $\ge 75\%$).
6. Inactive pills fill with `TRACK_MUTED`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p quota_tray_core`
Expected: PASS (all 20+ tests pass)

- [ ] **Step 5: Commit**

```bash
git add crates/core/src/icon.rs crates/core/tests/icon_render.rs
git commit -m "feat: implement 12-pill segmented radial ring tray icon with rounded capsule caps"
```

---

### Task 2: Instant Startup Icon & Menubar Polish (`tray.rs` & `poll_loop.rs`)

**Files:**
- Modify: `quota-tray/src-tauri/src/tray.rs`
- Modify: `quota-tray/src-tauri/src/poll_loop.rs`

- [ ] **Step 1: Set initial tray icon on app launch in `setup_tray`**

In `quota-tray/src-tauri/src/tray.rs`, pre-render the initial waiting icon so there is zero delay or blank space when Quota Tray launches:

```rust
let initial_state = quota_tray_core::TrayState {
    providers: vec![],
    shared_mascot_fill: None,
};
let initial_png = quota_tray_core::paint_tray_icon(&initial_state);
let initial_icon = tauri::image::Image::from_bytes(&initial_png).ok();

let mut builder = TrayIconBuilder::with_id("main")
    .tooltip("Quota Tray")
    .menu(&menu)
    .show_menu_on_left_click(false);

if let Some(icon) = initial_icon {
    builder = builder.icon(icon);
}
```

- [ ] **Step 2: Ensure pure icon mode (no text string in menubar)**

In `quota-tray/src-tauri/src/poll_loop.rs`, verify `tray.set_title(None)` so the menubar remains in pure icon mode as requested by the user.

- [ ] **Step 3: Verify build**

Run: `cargo check` in `quota-tray/src-tauri`
Expected: PASS with 0 errors

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/tray.rs src-tauri/src/poll_loop.rs
git commit -m "feat: pre-render initial tray icon at startup and ensure pure icon mode"
```

---

### Task 3: Build, Reinstall into `/Applications/` & System Verification

**Files:**
- Test & Bundle: All files in `quota-tray`

- [ ] **Step 1: Run complete test suite**

Run: `cargo test -p quota_tray_core`
Expected: All tests pass.

- [ ] **Step 2: Build release application**

Run: `cargo tauri build` in `quota-tray/src-tauri`
Expected: `Quota Tray.app` builds successfully.

- [ ] **Step 3: Reinstall to `/Applications/Quota Tray.app` and restart**

Copy bundle to `/Applications/Quota Tray.app`, kill existing instance, and relaunch.

- [ ] **Step 4: Commit and finalize**

```bash
git add .
git commit -m "chore: complete 12-pill segmented ring menubar tray icon redesign"
```
