//! Floating, always-on-top usage widget.
//!
//! A small borderless window that rests as a faint pill and grows into a full
//! card while the pointer is over it. Hover is polled natively rather than
//! read from the webview: WKWebView stops delivering pointer events once the
//! app is in the background, which is the widget's normal state.

use crate::widget_geometry::{
    anchor_on_screen, default_anchor, expansion_shift, HoverTracker, Rect,
};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use tauri::menu::CheckMenuItem;
use tauri::{
    AppHandle, Emitter, Manager, Monitor, PhysicalPosition, PhysicalSize, WebviewUrl,
    WebviewWindow, WebviewWindowBuilder, WindowEvent, Wry,
};

pub const WIDGET_LABEL: &str = "widget";

/// Logical sizes; keep in step with `WidgetApp.svelte`, which fills the window.
const COLLAPSED: (f64, f64) = (212.0, 64.0);
const EXPANDED: (f64, f64) = (336.0, 224.0);
const SCREEN_MARGIN: f64 = 12.0;
const DEFAULT_INSET: f64 = 16.0;
const CARD_RADIUS: f64 = 22.0;

const HOVER_POLL: Duration = Duration::from_millis(80);
/// Consecutive polls outside the window before it collapses (~320 ms).
const LEAVE_GRACE_TICKS: u8 = 4;
/// Moves caused by our own resize/reposition are not the user dragging.
const PROGRAMMATIC_MOVE_QUIET: Duration = Duration::from_millis(400);
const SAVE_THROTTLE: Duration = Duration::from_millis(400);
const PREFS_FILE: &str = "widget.json";

/// What survives a restart: where the user parked the widget, and whether it is shown.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
struct Prefs {
    /// Top-left of the collapsed pill, in physical pixels.
    anchor: Option<(i32, i32)>,
    visible: bool,
}

impl Default for Prefs {
    fn default() -> Self {
        Self {
            anchor: None,
            visible: true,
        }
    }
}

pub struct WidgetState {
    data_dir: PathBuf,
    prefs: Mutex<Prefs>,
    expanded: AtomicBool,
    /// How far (left, up) the expanded card sits from the anchor to stay on screen.
    shift: Mutex<(i32, i32)>,
    /// Ignore `Moved` events until this instant (they are our own doing).
    quiet_until: Mutex<Instant>,
    last_save: Mutex<Instant>,
    menu_item: Mutex<Option<CheckMenuItem<Wry>>>,
}

impl WidgetState {
    fn new(data_dir: PathBuf, prefs: Prefs) -> Self {
        let long_ago = Instant::now() - SAVE_THROTTLE;
        Self {
            data_dir,
            prefs: Mutex::new(prefs),
            expanded: AtomicBool::new(false),
            shift: Mutex::new((0, 0)),
            quiet_until: Mutex::new(long_ago),
            last_save: Mutex::new(long_ago),
            menu_item: Mutex::new(None),
        }
    }

    fn save(&self) {
        let prefs = *self.prefs.lock();
        if fs::create_dir_all(&self.data_dir).is_err() {
            return;
        }
        if let Ok(raw) = serde_json::to_string_pretty(&prefs) {
            let _ = fs::write(self.data_dir.join(PREFS_FILE), raw);
        }
    }
}

fn load_prefs(data_dir: &PathBuf) -> Prefs {
    fs::read_to_string(data_dir.join(PREFS_FILE))
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

fn work_area(monitor: &Monitor) -> Rect {
    let area = monitor.work_area();
    Rect::new(
        area.position.x,
        area.position.y,
        area.size.width as i32,
        area.size.height as i32,
    )
}

fn physical(size: (f64, f64), scale: f64) -> (i32, i32) {
    ((size.0 * scale).round() as i32, (size.1 * scale).round() as i32)
}

/// Builds the widget window (hidden if the user last hid it) and starts the
/// hover watcher. Call once from `setup`, before the tray is built.
pub fn init(app: &AppHandle, data_dir: PathBuf) -> tauri::Result<()> {
    let prefs = load_prefs(&data_dir);
    app.manage(WidgetState::new(data_dir, prefs));

    let window = WebviewWindowBuilder::new(app, WIDGET_LABEL, WebviewUrl::App("widget.html".into()))
        .title("Quota Tray Widget")
        .inner_size(COLLAPSED.0, COLLAPSED.1)
        .decorations(false)
        .resizable(false)
        .transparent(true)
        .shadow(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible_on_all_workspaces(true)
        // Showing the widget must not pull focus from whatever the user is typing in.
        .focused(false)
        .accept_first_mouse(true)
        .visible(prefs.visible)
        .build()?;

    place_initially(app, &window, prefs);

    let handle = app.clone();
    window.on_window_event(move |event| {
        if let WindowEvent::Moved(pos) = event {
            remember_position(&handle, (pos.x, pos.y));
        }
    });

    spawn_hover_watcher(app.clone());
    Ok(())
}

/// Hands the tray's "Show Widget" check item to the widget so it can keep it
/// in step when the widget is hidden from its own close button.
pub fn attach_menu_item(app: &AppHandle, item: CheckMenuItem<Wry>) {
    *app.state::<WidgetState>().menu_item.lock() = Some(item);
}

pub fn is_visible(app: &AppHandle) -> bool {
    app.state::<WidgetState>().prefs.lock().visible
}

fn place_initially(app: &AppHandle, window: &WebviewWindow, prefs: Prefs) {
    let scale = window.scale_factor().unwrap_or(1.0);
    let size = physical(COLLAPSED, scale);
    let areas: Vec<Rect> = app
        .available_monitors()
        .unwrap_or_default()
        .iter()
        .map(work_area)
        .collect();

    let anchor = prefs
        .anchor
        .filter(|a| anchor_on_screen(*a, &areas))
        .or_else(|| {
            let primary = app.primary_monitor().ok().flatten()?;
            Some(default_anchor(
                work_area(&primary),
                size,
                (DEFAULT_INSET * scale).round() as i32,
            ))
        });

    if let Some(anchor) = anchor {
        app.state::<WidgetState>().prefs.lock().anchor = Some(anchor);
        let _ = window.set_position(PhysicalPosition::new(anchor.0, anchor.1));
    }
}

/// Records where the user dropped the widget. The window may be sitting
/// shifted for an expanded card, so the stored anchor undoes that shift.
fn remember_position(app: &AppHandle, window_pos: (i32, i32)) {
    let state = app.state::<WidgetState>();
    if Instant::now() < *state.quiet_until.lock() {
        return;
    }
    let (sx, sy) = *state.shift.lock();
    state.prefs.lock().anchor = Some((window_pos.0 + sx, window_pos.1 + sy));

    let mut last = state.last_save.lock();
    if last.elapsed() >= SAVE_THROTTLE {
        *last = Instant::now();
        drop(last);
        state.save();
    }
}

fn spawn_hover_watcher(app: AppHandle) {
    std::thread::spawn(move || {
        let mut tracker = HoverTracker::new(LEAVE_GRACE_TICKS);
        loop {
            std::thread::sleep(HOVER_POLL);
            let Some(window) = app.get_webview_window(WIDGET_LABEL) else {
                continue;
            };
            if !window.is_visible().unwrap_or(false) {
                continue;
            }
            let inside = pointer_inside(&app, &window);
            let expanded = app.state::<WidgetState>().expanded.load(Ordering::SeqCst);
            if let Some(next) = tracker.observe(inside, expanded) {
                set_expanded(&app, &window, next);
            }
        }
    });
}

fn pointer_inside(app: &AppHandle, window: &WebviewWindow) -> bool {
    let (Ok(cursor), Ok(pos), Ok(size)) = (
        app.cursor_position(),
        window.outer_position(),
        window.outer_size(),
    ) else {
        return false;
    };
    Rect::new(pos.x, pos.y, size.width as i32, size.height as i32).contains(cursor.x, cursor.y)
}

/// Swaps between the resting pill and the full card: resizes the window,
/// nudges it back on screen if the card would spill off an edge, and tells the
/// page so it can fade between its two layers.
fn set_expanded(app: &AppHandle, window: &WebviewWindow, expanded: bool) {
    let state = app.state::<WidgetState>();
    let scale = window.scale_factor().unwrap_or(1.0);
    let Some(anchor) = state.prefs.lock().anchor.or_else(|| {
        window.outer_position().ok().map(|p| (p.x, p.y))
    }) else {
        return;
    };

    let size = physical(if expanded { EXPANDED } else { COLLAPSED }, scale);
    let shift = if expanded {
        let monitor = window
            .monitor_from_point(f64::from(anchor.0), f64::from(anchor.1))
            .ok()
            .flatten()
            .or_else(|| window.current_monitor().ok().flatten());
        monitor.map_or((0, 0), |m| {
            expansion_shift(
                anchor,
                size,
                work_area(&m),
                (SCREEN_MARGIN * scale).round() as i32,
            )
        })
    } else {
        (0, 0)
    };

    state.expanded.store(expanded, Ordering::SeqCst);
    *state.shift.lock() = shift;
    *state.quiet_until.lock() = Instant::now() + PROGRAMMATIC_MOVE_QUIET;

    apply_vibrancy(window, expanded);
    let _ = window.set_size(PhysicalSize::new(size.0 as u32, size.1 as u32));
    let _ = window.set_position(PhysicalPosition::new(anchor.0 - shift.0, anchor.1 - shift.1));
    let _ = app.emit_to(WIDGET_LABEL, "widget-hover", expanded);

    if !expanded {
        // The user may have dragged while expanded; settle the final spot on disk.
        state.save();
    }
}

/// Frosted glass behind the card on macOS. The resting pill stays plain so
/// the whole thing can fade to a faint ghost without a blurred slab left behind.
fn apply_vibrancy(window: &WebviewWindow, expanded: bool) {
    #[cfg(target_os = "macos")]
    {
        use tauri::utils::config::WindowEffectsConfig;
        use tauri::window::{Effect, EffectState, EffectsBuilder};

        let effects: Option<WindowEffectsConfig> = expanded.then(|| {
            EffectsBuilder::new()
                .effect(Effect::Popover)
                .state(EffectState::Active)
                .radius(CARD_RADIUS)
                .build()
        });
        let _ = window.set_effects(effects);
    }
    #[cfg(not(target_os = "macos"))]
    let _ = (window, expanded, CARD_RADIUS);
}

/// Shows or hides the widget and remembers the choice.
pub fn set_visible(app: &AppHandle, visible: bool) {
    let state = app.state::<WidgetState>();
    state.prefs.lock().visible = visible;
    state.save();

    if let Some(window) = app.get_webview_window(WIDGET_LABEL) {
        if visible {
            let _ = window.show();
        } else {
            set_expanded(app, &window, false);
            let _ = window.hide();
        }
    }
    sync_menu_item(app, visible);
}

pub fn toggle(app: &AppHandle) {
    set_visible(app, !is_visible(app));
}

fn sync_menu_item(app: &AppHandle, visible: bool) {
    if let Some(item) = app.state::<WidgetState>().menu_item.lock().as_ref() {
        let _ = item.set_checked(visible);
    }
}

#[tauri::command]
pub fn set_widget_visible(app: AppHandle, visible: bool) {
    set_visible(&app, visible);
}
