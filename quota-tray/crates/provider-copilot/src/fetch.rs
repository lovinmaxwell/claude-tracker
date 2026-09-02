use crate::headline::copilot_headline_percent;
use quota_tray_core::{FetchError, ProviderId, ProviderSnapshot, UsageWindow, WindowKind};
use secrecy::{ExposeSecret, SecretString};
use serde_json::Value;
use time::OffsetDateTime;

pub struct CopilotClient {
    pub base_url: String,
    pub http: reqwest::blocking::Client,
}

impl Default for CopilotClient {
    fn default() -> Self {
        Self {
            base_url: "https://api.github.com".into(),
            http: reqwest::blocking::Client::new(),
        }
    }
}

impl CopilotClient {
    pub fn fetch_user(&self, token: &SecretString) -> Result<ProviderSnapshot, FetchError> {
        let url = format!(
            "{}/copilot_internal/user",
            self.base_url.trim_end_matches('/')
        );
        let res = self
            .http
            .get(&url)
            .header(
                "Authorization",
                format!("Bearer {}", token.expose_secret()),
            )
            .header("Accept", "application/json")
            .header("X-GitHub-Api-Version", "2025-04-01")
            .header("User-Agent", "QuotaTray/0.1")
            .send()
            .map_err(|e| FetchError::Network(e.to_string()))?;

        if !res.status().is_success() {
            return Err(FetchError::Http {
                status: res.status().as_u16(),
                body: res.text().unwrap_or_default(),
            });
        }
        let body: Value = res.json().map_err(|e| FetchError::Parse(e.to_string()))?;
        parse_copilot_user(&body)
    }
}

pub fn parse_copilot_user(body: &Value) -> Result<ProviderSnapshot, FetchError> {
    let snapshots = body
        .get("quota_snapshots")
        .ok_or_else(|| FetchError::Parse("missing quota_snapshots".into()))?;
    let headline = copilot_headline_percent(snapshots);

    let mut windows = Vec::new();
    for (id, label) in [
        ("premium_interactions", "Premium"),
        ("chat", "Chat"),
        ("completions", "Completions"),
    ] {
        if let Some(bucket) = snapshots.get(id) {
            let unlimited = bucket.get("unlimited").and_then(|v| v.as_bool()) == Some(true);
            let used = if unlimited {
                None
            } else {
                bucket
                    .get("percent_remaining")
                    .and_then(|v| v.as_f64())
                    .map(|r| 100.0 - r)
            };
            windows.push(UsageWindow {
                id: id.into(),
                label: label.into(),
                kind: WindowKind::Percent { used },
                resets_at: None,
            });
        }
    }

    if windows.is_empty() {
        return Err(FetchError::Parse(
            "copilot_internal/user: empty quota_snapshots".into(),
        ));
    }

    Ok(ProviderSnapshot {
        provider: ProviderId::Copilot,
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

    #[tokio::test]
    async fn fetch_maps_premium_used() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/copilot_internal/user"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "copilot_plan": "individual_pro",
                "quota_snapshots": {
                    "premium_interactions": { "percent_remaining": 25.0, "unlimited": false },
                    "chat": { "unlimited": true },
                    "completions": { "unlimited": true }
                }
            })))
            .mount(&server)
            .await;

        let base = server.uri();
        let snap = tokio::task::spawn_blocking(move || {
            let client = CopilotClient {
                base_url: base,
                http: reqwest::blocking::Client::new(),
            };
            client.fetch_user(&SecretString::from("gho_x"))
        })
        .await
        .unwrap()
        .unwrap();

        assert_eq!(snap.provider, ProviderId::Copilot);
        assert_eq!(snap.headline_percent, Some(75.0));
    }
}
