use provider_claude::{fetch_claude_usage, ANTHROPIC_BETA_OAUTH, USAGE_PATH};
use quota_tray_core::{ClaudeHeadlineMetric, FetchError, ProviderId, WindowKind};
use serde_json::json;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn fetches_and_maps_five_hour_and_seven_day() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(USAGE_PATH))
        .and(header("Authorization", "Bearer sk-ant-oat-test"))
        .and(header("anthropic-beta", ANTHROPIC_BETA_OAUTH))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "five_hour": {
                "utilization": 42.0,
                "resets_at": "2099-01-01T00:00:00Z"
            },
            "seven_day": {
                "utilization": 13.0,
                "resets_at": "2099-01-07T00:00:00Z"
            }
        })))
        .mount(&server)
        .await;

    let snap = fetch_claude_usage(
        &server.uri(),
        "sk-ant-oat-test",
        ClaudeHeadlineMetric::Highest,
    )
    .await
    .expect("fetch");

    assert_eq!(snap.provider, ProviderId::Claude);
    assert!(!snap.stale);
    assert!(snap.error.is_none());
    assert_eq!(snap.headline_percent, Some(42.0)); // Highest = max(42, 13)

    let five = snap.windows.iter().find(|w| w.id == "five_hour").unwrap();
    let seven = snap.windows.iter().find(|w| w.id == "seven_day").unwrap();
    match five.kind {
        WindowKind::Percent { used } => assert_eq!(used, Some(42.0)),
        _ => panic!("expected percent"),
    }
    match seven.kind {
        WindowKind::Percent { used } => assert_eq!(used, Some(13.0)),
        _ => panic!("expected percent"),
    }

    let expected_five_reset =
        OffsetDateTime::parse("2099-01-01T00:00:00Z", &Rfc3339).expect("valid RFC3339");
    let expected_seven_reset =
        OffsetDateTime::parse("2099-01-07T00:00:00Z", &Rfc3339).expect("valid RFC3339");
    assert_eq!(five.resets_at, Some(expected_five_reset));
    assert_eq!(seven.resets_at, Some(expected_seven_reset));
}

#[tokio::test]
async fn five_hour_headline_metric() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(USAGE_PATH))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "five_hour": { "utilization": 10.0 },
            "seven_day": { "utilization": 90.0 }
        })))
        .mount(&server)
        .await;

    let snap = fetch_claude_usage(&server.uri(), "tok", ClaudeHeadlineMetric::FiveHour)
        .await
        .unwrap();
    assert_eq!(snap.headline_percent, Some(10.0));
}

#[tokio::test]
async fn seven_day_headline_metric() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(USAGE_PATH))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "five_hour": { "utilization": 10.0 },
            "seven_day": { "utilization": 90.0 }
        })))
        .mount(&server)
        .await;

    let snap = fetch_claude_usage(&server.uri(), "tok", ClaudeHeadlineMetric::SevenDay)
        .await
        .unwrap();
    assert_eq!(snap.headline_percent, Some(90.0));
}

#[tokio::test]
async fn highest_uses_only_present_window_when_other_missing() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(USAGE_PATH))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "five_hour": { "utilization": 55.0 }
        })))
        .mount(&server)
        .await;

    let snap = fetch_claude_usage(&server.uri(), "tok", ClaudeHeadlineMetric::Highest)
        .await
        .unwrap();
    assert_eq!(snap.headline_percent, Some(55.0));
    assert_eq!(snap.windows.len(), 1);
    assert_eq!(snap.windows[0].id, "five_hour");
}

#[tokio::test]
async fn null_utilization_maps_to_none_not_zero() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(USAGE_PATH))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "five_hour": { "utilization": null },
            "seven_day": { "utilization": 22.0 }
        })))
        .mount(&server)
        .await;

    let snap = fetch_claude_usage(&server.uri(), "tok", ClaudeHeadlineMetric::Highest)
        .await
        .unwrap();

    let five = snap.windows.iter().find(|w| w.id == "five_hour").unwrap();
    match five.kind {
        WindowKind::Percent { used } => assert_eq!(used, None),
        _ => panic!("expected percent"),
    }
    assert_eq!(snap.headline_percent, Some(22.0));

    let snap = fetch_claude_usage(&server.uri(), "tok", ClaudeHeadlineMetric::FiveHour)
        .await
        .unwrap();
    assert_eq!(snap.headline_percent, None);
}

#[tokio::test]
async fn malformed_json_returns_parse_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(USAGE_PATH))
        .respond_with(ResponseTemplate::new(200).set_body_string("not-json"))
        .mount(&server)
        .await;

    let err = fetch_claude_usage(&server.uri(), "tok", ClaudeHeadlineMetric::Highest)
        .await
        .unwrap_err();
    assert!(matches!(err, FetchError::Parse(_)), "{err}");
}

#[tokio::test]
async fn http_error_is_fetch_error_not_zeros() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(USAGE_PATH))
        .respond_with(ResponseTemplate::new(401).set_body_string("nope"))
        .mount(&server)
        .await;

    let err = fetch_claude_usage(&server.uri(), "bad", ClaudeHeadlineMetric::Highest)
        .await
        .unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("401") || msg.contains("http"), "{msg}");
}

#[tokio::test]
async fn missing_windows_yield_none_headline_not_zero() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(USAGE_PATH))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({})))
        .mount(&server)
        .await;

    let snap = fetch_claude_usage(&server.uri(), "tok", ClaudeHeadlineMetric::Highest)
        .await
        .unwrap();
    assert!(snap.windows.is_empty());
    assert_eq!(snap.headline_percent, None);
}
