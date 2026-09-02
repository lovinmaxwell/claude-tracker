use parking_lot::{Mutex, RwLock};
use quota_tray_core::{AppConfig, Poller, TrayState};
use std::sync::Arc;

pub struct AppState {
    pub tray_state: Arc<RwLock<TrayState>>,
    pub config: Arc<RwLock<AppConfig>>,
    /// `Poller::tick` needs `&mut self`; mutex matches the brief's shared Arc ownership.
    pub poller: Arc<Mutex<Poller>>,
}

impl AppState {
    pub fn new(config: AppConfig, poller: Poller) -> Self {
        Self {
            tray_state: Arc::new(RwLock::new(TrayState {
                providers: Vec::new(),
                shared_mascot_fill: None,
            })),
            config: Arc::new(RwLock::new(config)),
            poller: Arc::new(Mutex::new(poller)),
        }
    }
}
