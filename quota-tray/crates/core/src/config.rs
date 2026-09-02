use crate::types::ProviderId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum ClaudeHeadlineMetric {
    FiveHour,
    SevenDay,
    #[default]
    Highest,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppConfig {
    /// v1: default 60; settings UI clamps 60..=120
    pub poll_interval_secs: u64,
    pub enabled: Vec<ProviderId>,
    pub claude_headline: ClaudeHeadlineMetric,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            poll_interval_secs: 60,
            enabled: vec![ProviderId::Claude],
            claude_headline: ClaudeHeadlineMetric::Highest,
        }
    }
}

/// Clamp poll interval to the v1 allowed range.
pub fn clamp_poll_interval_secs(secs: u64) -> u64 {
    secs.clamp(60, 120)
}
