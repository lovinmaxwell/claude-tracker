use quota_tray_core::{
    CredentialError, Credentials, FetchError, Poller, Provider, ProviderId, ProviderSnapshot,
};
use secrecy::SecretString;
use std::sync::{Arc, Mutex};
use time::OffsetDateTime;

struct FakeProvider {
    id: ProviderId,
    /// Scripted results per tick index.
    results: Mutex<Vec<Result<ProviderSnapshot, FetchError>>>,
    calls: Arc<Mutex<usize>>,
}

impl FakeProvider {
    fn new(id: ProviderId, results: Vec<Result<ProviderSnapshot, FetchError>>) -> Self {
        Self {
            id,
            results: Mutex::new(results),
            calls: Arc::new(Mutex::new(0)),
        }
    }

    fn ok_snap(id: ProviderId, pct: f64) -> ProviderSnapshot {
        ProviderSnapshot {
            provider: id,
            fetched_at: OffsetDateTime::from_unix_timestamp(1_700_000_000).unwrap(),
            windows: vec![],
            headline_percent: Some(pct),
            stale: false,
            error: None,
        }
    }
}

impl Provider for FakeProvider {
    fn id(&self) -> ProviderId {
        self.id.clone()
    }

    fn credentials(&self) -> Result<Credentials, CredentialError> {
        Ok(Credentials {
            raw: SecretString::from("fake"),
        })
    }

    fn fetch(&self, _creds: &Credentials) -> Result<ProviderSnapshot, FetchError> {
        let mut calls = self.calls.lock().unwrap();
        *calls += 1;
        let mut q = self.results.lock().unwrap();
        if q.is_empty() {
            return Err(FetchError::Network("empty script".into()));
        }
        q.remove(0)
    }
}

#[test]
fn clamps_interval_on_construct() {
    let p = Poller::new(vec![], 5);
    assert_eq!(p.interval_secs(), 60);
    let p = Poller::new(vec![], 200);
    assert_eq!(p.interval_secs(), 120);
}

#[test]
fn first_success_sets_live_snapshot_and_mascot_max() {
    let claude = FakeProvider::new(
        ProviderId::Claude,
        vec![Ok(FakeProvider::ok_snap(ProviderId::Claude, 62.0))],
    );
    let cursor = FakeProvider::new(
        ProviderId::Cursor,
        vec![Ok(FakeProvider::ok_snap(ProviderId::Cursor, 41.0))],
    );
    let mut poller = Poller::new(vec![Box::new(claude), Box::new(cursor)], 60);
    let state = poller.tick();
    assert_eq!(state.providers.len(), 2);
    assert!(!state.providers[0].stale);
    assert_eq!(state.shared_mascot_fill, Some(62.0));
}

#[test]
fn one_provider_error_marks_stale_keeps_last_good_others_update() {
    let claude = FakeProvider::new(
        ProviderId::Claude,
        vec![
            Ok(FakeProvider::ok_snap(ProviderId::Claude, 50.0)),
            Err(FetchError::Network("down".into())),
        ],
    );
    let cursor = FakeProvider::new(
        ProviderId::Cursor,
        vec![
            Ok(FakeProvider::ok_snap(ProviderId::Cursor, 10.0)),
            Ok(FakeProvider::ok_snap(ProviderId::Cursor, 20.0)),
        ],
    );
    let mut poller = Poller::new(vec![Box::new(claude), Box::new(cursor)], 60);
    let _ = poller.tick();
    let state = poller.tick();

    let claude_snap = state
        .providers
        .iter()
        .find(|p| p.provider == ProviderId::Claude)
        .unwrap();
    assert!(claude_snap.stale);
    assert_eq!(claude_snap.headline_percent, Some(50.0)); // last good, not zeros
    assert!(claude_snap.error.as_deref().unwrap().contains("down"));

    let cursor_snap = state
        .providers
        .iter()
        .find(|p| p.provider == ProviderId::Cursor)
        .unwrap();
    assert!(!cursor_snap.stale);
    assert_eq!(cursor_snap.headline_percent, Some(20.0));

    // Claude stale → excluded from max; healthy Cursor 20 wins
    assert_eq!(state.shared_mascot_fill, Some(20.0));
}

#[test]
fn never_had_success_stays_none_headline_not_fake_zero() {
    let claude = FakeProvider::new(
        ProviderId::Claude,
        vec![Err(FetchError::Network("nope".into()))],
    );
    let mut poller = Poller::new(vec![Box::new(claude)], 60);
    let state = poller.tick();
    let snap = &state.providers[0];
    assert!(snap.stale);
    assert_eq!(snap.headline_percent, None);
    assert_eq!(state.shared_mascot_fill, None);
}
