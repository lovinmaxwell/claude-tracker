use crate::types::ProviderId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum ClaudeHeadlineMetric {
    FiveHour,
    SevenDay,
    #[default]
    Highest,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
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

impl AppConfig {
    pub fn clamp_poll_interval(&mut self) {
        self.poll_interval_secs = clamp_poll_interval_secs(self.poll_interval_secs);
    }

    pub fn sanitized(mut self) -> Self {
        self.clamp_poll_interval();
        self.enabled.sort_by_key(|id| match id {
            ProviderId::Claude => 0,
            ProviderId::Cursor => 1,
            ProviderId::Copilot => 2,
            ProviderId::OpenAI => 3,
        });
        self.enabled.dedup();
        self
    }
}

/// Clamp poll interval to the v1 allowed range.
pub fn clamp_poll_interval_secs(secs: u64) -> u64 {
    secs.clamp(60, 120)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamp_poll_interval_rejects_below_60_and_above_120() {
        let mut low = AppConfig {
            poll_interval_secs: 15,
            ..AppConfig::default()
        };
        low.clamp_poll_interval();
        assert_eq!(low.poll_interval_secs, 60);

        let mut high = AppConfig {
            poll_interval_secs: 999,
            ..AppConfig::default()
        };
        high.clamp_poll_interval();
        assert_eq!(high.poll_interval_secs, 120);
    }

    #[test]
    fn sanitized_dedups_and_orders_enabled() {
        let cfg = AppConfig {
            poll_interval_secs: 30,
            enabled: vec![
                ProviderId::Copilot,
                ProviderId::Claude,
                ProviderId::Claude,
                ProviderId::Cursor,
            ],
            claude_headline: ClaudeHeadlineMetric::FiveHour,
        }
        .sanitized();
        assert_eq!(cfg.poll_interval_secs, 60);
        assert_eq!(
            cfg.enabled,
            vec![
                ProviderId::Claude,
                ProviderId::Cursor,
                ProviderId::Copilot,
            ]
        );
    }
}
