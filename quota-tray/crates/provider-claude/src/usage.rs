use quota_tray_core::{
    ClaudeHeadlineMetric, FetchError, ProviderId, ProviderSnapshot, UsageWindow, WindowKind,
};
use serde::Deserialize;
use std::sync::OnceLock;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

static HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

fn http_client() -> &'static reqwest::Client {
    HTTP_CLIENT.get_or_init(reqwest::Client::new)
}

pub const DEFAULT_API_BASE: &str = "https://api.anthropic.com";
pub const USAGE_PATH: &str = "/api/oauth/usage";
pub const ANTHROPIC_BETA_OAUTH: &str = "oauth-2025-04-20";

#[derive(Debug, Deserialize)]
struct UsageResponse {
    five_hour: Option<WindowDto>,
    seven_day: Option<WindowDto>,
}

#[derive(Debug, Deserialize)]
struct WindowDto {
    utilization: Option<f64>,
    resets_at: Option<String>,
}

fn parse_resets(raw: &Option<String>) -> Option<OffsetDateTime> {
    raw.as_ref()
        .and_then(|s| OffsetDateTime::parse(s, &Rfc3339).ok())
}

fn clamp_used(v: f64) -> f64 {
    v.clamp(0.0, 100.0)
}

fn map_window(id: &str, label: &str, dto: WindowDto) -> UsageWindow {
    UsageWindow {
        id: id.to_string(),
        label: label.to_string(),
        kind: WindowKind::Percent {
            used: dto.utilization.map(clamp_used),
        },
        resets_at: parse_resets(&dto.resets_at),
    }
}

fn percent_used(window: &UsageWindow) -> Option<f64> {
    match window.kind {
        WindowKind::Percent { used } => used,
        _ => None,
    }
}

fn headline_from_windows(
    windows: &[UsageWindow],
    metric: ClaudeHeadlineMetric,
) -> Option<f64> {
    let five = windows.iter().find(|w| w.id == "five_hour").and_then(percent_used);
    let seven = windows.iter().find(|w| w.id == "seven_day").and_then(percent_used);
    match metric {
        ClaudeHeadlineMetric::FiveHour => five,
        ClaudeHeadlineMetric::SevenDay => seven,
        ClaudeHeadlineMetric::Highest => match (five, seven) {
            (Some(a), Some(b)) => Some(a.max(b)),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        },
    }
}

/// GET `{base_url}/api/oauth/usage` with Bearer + anthropic-beta oauth header.
pub async fn fetch_claude_usage(
    base_url: &str,
    access_token: &str,
    headline: ClaudeHeadlineMetric,
) -> Result<ProviderSnapshot, FetchError> {
    let url = format!("{}{}", base_url.trim_end_matches('/'), USAGE_PATH);
    let response = http_client()
        .get(&url)
        .header("Authorization", format!("Bearer {access_token}"))
        .header("anthropic-beta", ANTHROPIC_BETA_OAUTH)
        .send()
        .await
        .map_err(|e| FetchError::Network(e.to_string()))?;

    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|e| FetchError::Network(e.to_string()))?;

    if !status.is_success() {
        return Err(FetchError::Http {
            status: status.as_u16(),
            body,
        });
    }

    let parsed: UsageResponse =
        serde_json::from_str(&body).map_err(|e| FetchError::Parse(e.to_string()))?;

    let mut windows = Vec::new();
    if let Some(dto) = parsed.five_hour {
        windows.push(map_window("five_hour", "5-hour", dto));
    }
    if let Some(dto) = parsed.seven_day {
        windows.push(map_window("seven_day", "7-day", dto));
    }

    let headline_percent = headline_from_windows(&windows, headline);

    Ok(ProviderSnapshot {
        provider: ProviderId::Claude,
        fetched_at: OffsetDateTime::now_utc(),
        windows,
        headline_percent,
        stale: false,
        error: None,
    })
}
