use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, PhysicalPosition, Position, Rect, Runtime, Size, WebviewUrl,
    WebviewWindow, WebviewWindowBuilder,
};

const PANEL_WIDTH: f64 = 360.0;
const PANEL_HEIGHT: f64 = 520.0;
const PANEL_GAP: f64 = 4.0;

pub fn setup_tray<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&quit])?;

    let _tray = TrayIconBuilder::with_id("main")
        .tooltip("Quota Tray")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            if event.id.as_ref() == "quit" {
                app.exit(0);
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                rect,
                ..
            } = event
            {
                let app = tray.app_handle();
                toggle_panel(app, Some(rect));
            }
        })
        .build(app)?;

    Ok(())
}

fn toggle_panel<R: Runtime>(app: &AppHandle<R>, tray_rect: Option<Rect>) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            apply_native_popover(&window);
            if let Some(rect) = tray_rect.as_ref() {
                position_near_tray(&window, rect);
            }
            let _ = window.show();
            let _ = window.set_focus();
        }
        return;
    }

    if let Ok(window) = WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
        .title("Quota Tray")
        .inner_size(PANEL_WIDTH, PANEL_HEIGHT)
        .decorations(false)
        .resizable(false)
        .transparent(true)
        .shadow(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(false)
        .build()
    {
        apply_native_popover(&window);
        if let Some(rect) = tray_rect.as_ref() {
            position_near_tray(&window, rect);
        }
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn apply_native_popover<R: Runtime>(window: &WebviewWindow<R>) {
    let _ = window.set_shadow(true);
    #[cfg(target_os = "macos")]
    {
        use tauri::window::{Effect, EffectState, EffectsBuilder};
        let _ = window.set_effects(
            EffectsBuilder::new()
                .effect(Effect::Popover)
                .state(EffectState::Active)
                .radius(12.0)
                .build(),
        );
    }
}

fn position_near_tray<R: Runtime>(window: &WebviewWindow<R>, tray_rect: &Rect) {
    let scale = window.scale_factor().unwrap_or(1.0);
    let icon_pos = match &tray_rect.position {
        Position::Physical(p) => PhysicalPosition::new(p.x as f64, p.y as f64),
        Position::Logical(p) => PhysicalPosition::new(p.x * scale, p.y * scale),
    };
    let icon_size = match &tray_rect.size {
        Size::Physical(s) => (s.width as f64, s.height as f64),
        Size::Logical(s) => (s.width * scale, s.height * scale),
    };

    let win_size = window
        .outer_size()
        .map(|s| (s.width as f64, s.height as f64))
        .unwrap_or((PANEL_WIDTH * scale, PANEL_HEIGHT * scale));

    // Center under the menubar icon, drop just below it (macOS tray is top).
    let mut x = icon_pos.x + icon_size.0 / 2.0 - win_size.0 / 2.0;
    let mut y = icon_pos.y + icon_size.1 + PANEL_GAP;

    if let Ok(Some(monitor)) = window.current_monitor() {
        let mpos = monitor.position();
        let msize = monitor.size();
        let left = f64::from(mpos.x);
        let top = f64::from(mpos.y);
        let right = left + f64::from(msize.width);
        let bottom = top + f64::from(msize.height);

        let max_x = (right - win_size.0 - 8.0).max(left + 8.0);
        x = x.clamp(left + 8.0, max_x);

        if y + win_size.1 > bottom - 8.0 {
            y = (icon_pos.y - win_size.1 - PANEL_GAP).max(top + 8.0);
        }
    }

    let _ = window.set_position(Position::Physical(PhysicalPosition::new(
        x.round() as i32,
        y.round() as i32,
    )));
}
