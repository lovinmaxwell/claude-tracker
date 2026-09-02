use quota_tray_core::{
    aggregate_mascot_fill, clamp_poll_interval_secs, ProviderId, ProviderSnapshot,
};
use time::OffsetDateTime;

fn snap(
    provider: ProviderId,
    headline: Option<f64>,
    stale: bool,
    error: Option<&str>,
) -> ProviderSnapshot {
    ProviderSnapshot {
        provider,
        fetched_at: OffsetDateTime::UNIX_EPOCH,
        windows: vec![],
        headline_percent: headline,
        stale,
        error: error.map(str::to_string),
    }
}

#[test]
fn max_among_healthy_some_headlines() {
    let providers = vec![
        snap(ProviderId::Claude, Some(62.0), false, None),
        snap(ProviderId::Cursor, Some(41.0), false, None),
        snap(ProviderId::Copilot, Some(10.0), false, None),
    ];
    assert_eq!(aggregate_mascot_fill(&providers, None), Some(62.0));
}

#[test]
fn excludes_stale_error_and_none_headline() {
    let providers = vec![
        snap(ProviderId::Claude, Some(90.0), true, None), // stale
        snap(ProviderId::Cursor, Some(80.0), false, Some("boom")), // error
        snap(ProviderId::Copilot, None, false, None), // no headline
        snap(ProviderId::OpenAI, Some(55.0), false, None),
    ];
    assert_eq!(aggregate_mascot_fill(&providers, Some(12.0)), Some(55.0));
}

#[test]
fn empty_healthy_reuses_last_successful() {
    let providers = vec![
        snap(ProviderId::Claude, Some(90.0), true, Some("fail")),
    ];
    assert_eq!(aggregate_mascot_fill(&providers, Some(33.0)), Some(33.0));
}

#[test]
fn empty_healthy_and_no_last_returns_none_never_zero() {
    let providers = vec![
        snap(ProviderId::Claude, None, true, Some("waiting")),
    ];
    assert_eq!(aggregate_mascot_fill(&providers, None), None);
}

#[test]
fn clamp_poll_interval_secs_bounds() {
    assert_eq!(clamp_poll_interval_secs(1), 60);
    assert_eq!(clamp_poll_interval_secs(60), 60);
    assert_eq!(clamp_poll_interval_secs(90), 90);
    assert_eq!(clamp_poll_interval_secs(120), 120);
    assert_eq!(clamp_poll_interval_secs(999), 120);
}
