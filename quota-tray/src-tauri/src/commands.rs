use crate::state::AppState;
use quota_tray_core::{AppConfig, TrayState};
use tauri::{AppHandle, Emitter, Manager, State};

#[tauri::command]
pub fn get_tray_state(state: State<'_, AppState>) -> Result<TrayState, String> {
    Ok(state.tray_state.read().clone())
}

#[tauri::command]
pub fn get_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    Ok(state.config.read().clone())
}

#[tauri::command]
pub fn set_config(
    app: AppHandle,
    state: State<'_, AppState>,
    mut config: AppConfig,
) -> Result<AppConfig, String> {
    config = config.sanitized();
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    crate::config_store::save(data_dir, &config)?;
    *state.config.write() = config.clone();

    let tray = {
        let mut poller = state.poller.lock();
        poller.replace_providers(crate::providers::from_config(&config))
    };
    *state.tray_state.write() = tray;
    let _ = app.emit("tray-state-updated", ());

    Ok(config)
}

#[cfg(test)]
mod tests {
    use quota_tray_core::{
        ProviderId, ProviderSnapshot, TrayState, UsageWindow, WindowKind,
    };
    use time::OffsetDateTime;

    #[test]
    fn tray_state_serializes_for_ipc() {
        let state = TrayState {
            providers: vec![ProviderSnapshot {
                provider: ProviderId::Claude,
                fetched_at: OffsetDateTime::from_unix_timestamp(1_700_000_000).unwrap(),
                windows: vec![UsageWindow {
                    id: "five_hour".into(),
                    label: "5-hour".into(),
                    kind: WindowKind::Percent { used: Some(42.0) },
                    resets_at: None,
                }],
                headline_percent: Some(42.0),
                stale: false,
                error: None,
            }],
            shared_mascot_fill: Some(42.0),
        };
        let v = serde_json::to_value(&state).expect("serialize");
        assert_eq!(v["shared_mascot_fill"], 42.0);
        assert_eq!(v["providers"][0]["provider"], "Claude");
        assert!(v["providers"][0]["headline_percent"].as_f64().unwrap() > 40.0);
        assert_eq!(v["providers"][0]["stale"], false);
    }
}
