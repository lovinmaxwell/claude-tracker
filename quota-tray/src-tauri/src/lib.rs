mod commands;
mod config_store;
mod poll_loop;
mod providers;
mod state;
mod tray;

use quota_tray_core::{AppConfig, Poller};
use state::AppState;
use std::sync::Arc;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let config = AppConfig::default();
    let poller = Poller::new(
        providers::from_config(&config),
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
        .invoke_handler(tauri::generate_handler![
            commands::get_tray_state,
            commands::get_config,
            commands::set_config,
        ])
        .setup(move |app| {
            let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
            let loaded = config_store::load(data_dir);
            {
                *app_state.config.write() = loaded.clone();
                let mut poller = app_state.poller.lock();
                let tray = poller.replace_providers(providers::from_config(&loaded));
                *app_state.tray_state.write() = tray;
            }
            tray::setup_tray(app.handle())?;
            poll_loop::spawn_poll_loop(app.handle().clone(), app_state.clone());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Quota Tray");
}
