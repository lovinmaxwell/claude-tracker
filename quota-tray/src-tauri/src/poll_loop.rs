use crate::state::AppState;
use quota_tray_core::{paint_tray_icon, Poller};
use std::sync::Arc;
use std::time::Duration;
use tauri::tray::TrayIcon;
use tauri::{AppHandle, Emitter};

pub fn spawn_poll_loop(app: AppHandle, state: Arc<AppState>) {
    tauri::async_runtime::spawn(async move {
        loop {
            let interval_secs = {
                let cfg = state.config.read();
                cfg.poll_interval_secs.clamp(60, 120)
            };
            // Clone provider handles under a brief lock; network fetch runs unlocked.
            let providers = {
                let poller = state.poller.lock();
                poller.providers()
            };
            let results = Poller::fetch_providers(&providers);
            let tick = {
                let mut poller = state.poller.lock();
                poller.apply_fetch_results(results)
            };
            {
                let mut guard = state.tray_state.write();
                *guard = tick;
            }
            if let Some(tray) = app.tray_by_id("main") {
                apply_tray_icon(&tray, &state.tray_state.read());
            }
            let _ = app.emit("tray-state-updated", ());
            tokio::time::sleep(Duration::from_secs(interval_secs)).await;
        }
    });
}

fn apply_tray_icon(tray: &TrayIcon, state: &quota_tray_core::TrayState) {
    let png = paint_tray_icon(state);
    if let Ok(icon) = tauri::image::Image::from_bytes(&png) {
        let _ = tray.set_icon(Some(icon));
    }
    let tooltip = state
        .providers
        .iter()
        .map(|p| {
            let name = match p.provider {
                quota_tray_core::ProviderId::Claude => "Claude",
                quota_tray_core::ProviderId::Cursor => "Cursor",
                quota_tray_core::ProviderId::Copilot => "Copilot",
                quota_tray_core::ProviderId::OpenAI => "OpenAI",
            };
            if p.stale {
                format!("{name} stale")
            } else if let Some(h) = p.headline_percent {
                format!("{name} {h:.0}%")
            } else {
                format!("{name} —")
            }
        })
        .collect::<Vec<_>>()
        .join(" · ");
    let _ = tray.set_tooltip(Some(&tooltip));
}
