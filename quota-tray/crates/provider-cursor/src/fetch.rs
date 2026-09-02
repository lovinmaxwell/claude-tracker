use crate::headline::cursor_headline_percent;
use quota_tray_core::{FetchError, ProviderId, ProviderSnapshot, UsageWindow, WindowKind};
use secrecy::{ExposeSecret, SecretString};
use serde_json::Value;
use time::OffsetDateTime;

pub const DEFAULT_API_BASE: &str = "https://api2.cursor.sh";
pub const USAGE_PATH: &str = "/aiserver.v1.DashboardService/GetCurrentPeriodUsage";

pub struct CursorClient {
    pub base_url: String,
    pub http: reqwest::blocking::Client,
}

impl Default for CursorClient {
    fn default() -> Self {
        Self {
            base_url: DEFAULT_API_BASE.into(),
            http: reqwest::blocking::Client::new(),
        }
    }
}

impl CursorClient {
    pub fn fetch_usage(&self, token: &SecretString) -> Result<ProviderSnapshot, FetchError> {
        let url = format!(
            "{}{}",
            self.base_url.trim_end_matches('/'),
            USAGE_PATH
        );
        let res = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {}", token.expose_secret()))
            .header("Content-Type", "application/json")
            .body("{}")
            .send()
            .map_err(|e| FetchError::Network(e.to_string()))?;

        if !res.status().is_success() {
            return Err(FetchError::Http {
                status: res.status().as_u16(),
                body: res.text().unwrap_or_default(),
            });
        }

        let body: Value = res
            .json()
            .map_err(|e| FetchError::Parse(e.to_string()))?;
        parse_usage_body(&body)
    }
}

pub fn parse_usage_body(body: &Value) -> Result<ProviderSnapshot, FetchError> {
    let plan = body
        .get("planUsage")
        .cloned()
        .unwrap_or_else(|| body.clone());

    let headline = cursor_headline_percent(&plan);
    let mut windows = Vec::new();

    if let Some(auto) = plan.get("autoPercentUsed").and_then(|v| v.as_f64()) {
        windows.push(UsageWindow {
            id: "auto".into(),
            label: "Auto / Composer".into(),
            kind: WindowKind::Percent { used: Some(auto) },
            resets_at: None,
        });
    }
    if let Some(api) = plan.get("apiPercentUsed").and_then(|v| v.as_f64()) {
        windows.push(UsageWindow {
            id: "api".into(),
            label: "API".into(),
            kind: WindowKind::Percent { used: Some(api) },
            resets_at: None,
        });
    }
    if windows.is_empty() {
        windows.push(UsageWindow {
            id: "period".into(),
            label: "Current period".into(),
            kind: WindowKind::Percent { used: headline },
            resets_at: None,
        });
    }

    // Defensive: empty object with no usable fields is a parse failure, not fake zeros.
    if headline.is_none()
        && plan.get("totalPercentUsed").is_none()
        && plan.get("autoPercentUsed").is_none()
        && plan.get("apiPercentUsed").is_none()
        && plan.get("totalSpend").is_none()
    {
        return Err(FetchError::Parse(
            "GetCurrentPeriodUsage: no planUsage percent/spend fields".into(),
        ));
    }

    Ok(ProviderSnapshot {
        provider: ProviderId::Cursor,
        fetched_at: OffsetDateTime::now_utc(),
        windows,
        headline_percent: headline,
        stale: false,
        error: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn parse_rejects_empty_plan() {
        let err = parse_usage_body(&json!({})).unwrap_err();
        assert!(matches!(err, FetchError::Parse(_)), "{err}");
    }

    #[test]
    fn parse_maps_spend_only_to_period_window() {
        let snap = parse_usage_body(&json!({
            "planUsage": { "totalSpend": 250.0, "limit": 1000.0 }
        }))
        .unwrap();
        assert_eq!(snap.headline_percent, Some(25.0));
        assert_eq!(snap.windows.len(), 1);
        assert_eq!(snap.windows[0].id, "period");
    }

    #[tokio::test]
    async fn fetch_maps_plan_usage() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path(USAGE_PATH))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "planUsage": {
                    "totalPercentUsed": 41.5,
                    "autoPercentUsed": 20.0,
                    "apiPercentUsed": 41.5
                }
            })))
            .mount(&server)
            .await;

        let base = server.uri();
        let snap = tokio::task::spawn_blocking(move || {
            let client = CursorClient {
                base_url: base,
                http: reqwest::blocking::Client::new(),
            };
            client.fetch_usage(&SecretString::from("tok"))
        })
        .await
        .unwrap()
        .unwrap();

        assert_eq!(snap.provider, ProviderId::Cursor);
        assert_eq!(snap.headline_percent, Some(41.5));
        assert!(!snap.stale);
        assert!(snap.windows.len() >= 2);
    }
}
