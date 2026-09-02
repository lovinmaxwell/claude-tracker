use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// Panel / tray copy when an enabled provider has never been polled.
pub const WAITING_FIRST_READING: &str = "Waiting for the first reading";

/// Stable id used in config, tray segments, and UI chips.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProviderId {
    Claude,
    Cursor,
    Copilot,
    OpenAI,
}

impl ProviderId {
    /// Chip / segment color only (not mascot fill). Cream/terracotta theme — no purple.
    pub fn chip_hex(&self) -> &'static str {
        match self {
            ProviderId::Claude => "#C96442",
            ProviderId::Cursor => "#5A7A6A",
            ProviderId::Copilot => "#3D5A80",
            ProviderId::OpenAI => "#6B5D50",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            ProviderId::Claude => "Claude",
            ProviderId::Cursor => "Cursor",
            ProviderId::Copilot => "Copilot",
            ProviderId::OpenAI => "OpenAI",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderInfo {
    pub id: ProviderId,
    pub display_name: &'static str,
    /// Chip / segment color only (not mascot fill).
    pub chip_color: &'static str,
}

impl ProviderInfo {
    pub fn for_id(id: ProviderId) -> Self {
        Self {
            display_name: id.display_name(),
            chip_color: id.chip_hex(),
            id,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UsageWindow {
    pub id: String,
    pub label: String,
    pub kind: WindowKind,
    pub resets_at: Option<OffsetDateTime>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum WindowKind {
    /// 0.0..=100.0 used; None means "vendor did not say"
    Percent { used: Option<f64> },
    Currency {
        used: Option<f64>,
        limit: Option<f64>,
        code: String,
    },
    Count {
        used: Option<u64>,
        limit: Option<u64>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderSnapshot {
    pub provider: ProviderId,
    pub fetched_at: OffsetDateTime,
    pub windows: Vec<UsageWindow>,
    /// Drives tray segment + contribution to shared mascot fill (% used).
    pub headline_percent: Option<f64>,
    pub stale: bool,
    pub error: Option<String>,
}

impl ProviderSnapshot {
    /// Healthy = not stale, no error, and headline_percent is Some.
    pub fn is_healthy(&self) -> bool {
        !self.stale && self.error.is_none() && self.headline_percent.is_some()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrayState {
    /// Enabled only, stable order.
    pub providers: Vec<ProviderSnapshot>,
    /// max % used among healthy; None if none healthy and no last success.
    pub shared_mascot_fill: Option<f64>,
}
