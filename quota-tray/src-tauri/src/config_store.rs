use quota_tray_core::AppConfig;
use std::fs;
use std::path::PathBuf;

pub fn config_path(app_data: PathBuf) -> PathBuf {
    app_data.join("config.json")
}

pub fn load(app_data: PathBuf) -> AppConfig {
    let path = config_path(app_data);
    match fs::read_to_string(&path) {
        Ok(raw) => serde_json::from_str::<AppConfig>(&raw)
            .unwrap_or_default()
            .sanitized(),
        Err(_) => AppConfig::default(),
    }
}

pub fn save(app_data: PathBuf, config: &AppConfig) -> Result<(), String> {
    fs::create_dir_all(&app_data).map_err(|e| e.to_string())?;
    let path = config_path(app_data);
    let raw = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(path, raw).map_err(|e| e.to_string())
}
