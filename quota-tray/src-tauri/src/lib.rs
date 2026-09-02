mod commands;
mod poll_loop;
mod state;
mod tray;

use provider_claude::ClaudeProvider;
use quota_tray_core::{AppConfig, ClaudeHeadlineMetric, Poller, ProviderId};
use state::AppState;
use std::sync::Arc;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let config = AppConfig {
        poll_interval_secs: 60,
        enabled: vec![ProviderId::Claude],
        claude_headline: ClaudeHeadlineMetric::Highest,
    };
    // Actual Poller::new takes interval secs (not AppConfig); enabled list is
    // applied by which providers we register here.
    let poller = Poller::new(
        vec![Box::new(ClaudeProvider::default())],
        config.poll_interval_secs,
    );
    let app_state = Arc::new(AppState::new(config, poller));

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            tray_state: app_state.tray_state.clone(),
            config: app_state.config.clone(),
            poller: app_state.poller.clone(),
        })
        .invoke_handler(tauri::generate_handler![commands::get_tray_state])
        .setup(move |app| {
            tray::setup_tray(app.handle())?;
            poll_loop::spawn_poll_loop(app.handle().clone(), app_state.clone());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Quota Tray");
}
