//! Shared types, aggregation, and poller for Quota Tray.

mod aggregate;
mod config;
mod provider;
mod types;

pub use aggregate::aggregate_mascot_fill;
pub use config::{clamp_poll_interval_secs, AppConfig, ClaudeHeadlineMetric};
pub use provider::{CredentialError, Credentials, FetchError, Provider};
pub use types::{
    ProviderId, ProviderInfo, ProviderSnapshot, TrayState, UsageWindow, WindowKind,
};
