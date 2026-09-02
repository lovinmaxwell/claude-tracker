use crate::aggregate::aggregate_mascot_fill;
use crate::config::clamp_poll_interval_secs;
use crate::provider::Provider;
use crate::types::{ProviderId, ProviderSnapshot, TrayState};
use std::collections::HashMap;
use std::sync::Arc;
use time::OffsetDateTime;

pub struct Poller {
    providers: Arc<Vec<Arc<dyn Provider>>>,
    interval_secs: u64,
    last: HashMap<ProviderId, ProviderSnapshot>,
    last_mascot: Option<f64>,
}

impl Poller {
    pub fn new(providers: Vec<Box<dyn Provider>>, poll_interval_secs: u64) -> Self {
        Self {
            providers: Arc::new(
                providers
                    .into_iter()
                    .map(|provider| Arc::from(provider) as Arc<dyn Provider>)
                    .collect(),
            ),
            interval_secs: clamp_poll_interval_secs(poll_interval_secs),
            last: HashMap::new(),
            last_mascot: None,
        }
    }

    pub fn interval_secs(&self) -> u64 {
        self.interval_secs
    }

    pub fn providers(&self) -> Arc<Vec<Arc<dyn Provider>>> {
        Arc::clone(&self.providers)
    }

    /// Swap the registered provider set (e.g. after Settings save). Drops cached
    /// snapshots for providers that are no longer registered and returns a
    /// tray state rebuilt from remaining cache (no network).
    pub fn replace_providers(&mut self, providers: Vec<Box<dyn Provider>>) -> TrayState {
        self.providers = Arc::new(
            providers
                .into_iter()
                .map(|provider| Arc::from(provider) as Arc<dyn Provider>)
                .collect(),
        );
        let keep: std::collections::HashSet<ProviderId> =
            self.providers.iter().map(|p| p.id()).collect();
        self.last.retain(|id, _| keep.contains(id));
        self.tray_from_cache()
    }

    fn tray_from_cache(&self) -> TrayState {
        let providers: Vec<ProviderSnapshot> = self
            .providers
            .iter()
            .filter_map(|p| self.last.get(&p.id()).cloned())
            .collect();
        let shared = aggregate_mascot_fill(&providers, self.last_mascot);
        TrayState {
            providers,
            shared_mascot_fill: shared,
        }
    }

    /// Network fetch only — safe to run without holding a poller mutex.
    pub fn fetch_providers(
        providers: &[Arc<dyn Provider>],
    ) -> Vec<(ProviderId, Result<ProviderSnapshot, String>)> {
        std::thread::scope(|scope| {
            let mut handles = Vec::new();
            for provider in providers {
                let provider = Arc::clone(provider);
                handles.push(scope.spawn(move || {
                    let id = provider.id();
                    let outcome = (|| {
                        let creds = provider.credentials().map_err(|e| e.to_string())?;
                        provider.fetch(&creds).map_err(|e| e.to_string())
                    })();
                    (id, outcome)
                }));
            }
            handles
                .into_iter()
                .map(|h| h.join().expect("provider thread"))
                .collect()
        })
    }

    /// Merge fetch results into cached snapshots and build tray state.
    pub fn apply_fetch_results(
        &mut self,
        results: Vec<(ProviderId, Result<ProviderSnapshot, String>)>,
    ) -> TrayState {
        for (id, outcome) in results {
            match outcome {
                Ok(mut snap) => {
                    snap.stale = false;
                    snap.error = None;
                    self.last.insert(id, snap);
                }
                Err(err) => {
                    if let Some(prev) = self.last.get_mut(&id) {
                        prev.stale = true;
                        prev.error = Some(err);
                    } else {
                        self.last.insert(
                            id.clone(),
                            ProviderSnapshot {
                                provider: id,
                                fetched_at: OffsetDateTime::now_utc(),
                                windows: vec![],
                                headline_percent: None,
                                stale: true,
                                error: Some(err),
                            },
                        );
                    }
                }
            }
        }

        let providers: Vec<ProviderSnapshot> = self
            .providers
            .iter()
            .filter_map(|p| self.last.get(&p.id()).cloned())
            .collect();

        let shared = aggregate_mascot_fill(&providers, self.last_mascot);
        if providers.iter().any(|p| p.is_healthy()) {
            self.last_mascot = shared;
        }

        TrayState {
            providers,
            shared_mascot_fill: shared,
        }
    }

    /// One parallel poll cycle. Failures are independent (stale-on-error).
    pub fn tick(&mut self) -> TrayState {
        let results = Self::fetch_providers(&self.providers);
        self.apply_fetch_results(results)
    }
}
