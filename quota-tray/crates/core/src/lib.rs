//! Shared types, aggregation, and poller for Quota Tray.

mod aggregate;
mod config;
mod icon;
mod poller;
mod provider;
mod types;

pub use aggregate::aggregate_mascot_fill;
pub use config::{clamp_poll_interval_secs, AppConfig, ClaudeHeadlineMetric};
pub use icon::{
    paint_provider_segments, paint_tray_icon, render_mascot_png, render_mascot_rgba,
    render_tray_rgba, IconError, RgbaColor, CREAM, MASCOT_ART, MASCOT_GRID, TERRACOTTA,
};
pub use poller::{waiting_snapshot, Poller};
pub use provider::{CredentialError, Credentials, FetchError, Provider};
pub use types::{
    ProviderId, ProviderInfo, ProviderSnapshot, TrayState, UsageWindow, WindowKind,
    WAITING_FIRST_READING,
};
