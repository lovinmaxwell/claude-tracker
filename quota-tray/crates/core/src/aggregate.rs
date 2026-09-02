use crate::types::ProviderSnapshot;

/// `shared_mascot_fill = max(headline_percent)` among healthy providers.
/// If the healthy set is empty, reuse `last_successful` (muted in UI) or `None`.
/// Never invents a live `0.0`.
pub fn aggregate_mascot_fill(
    providers: &[ProviderSnapshot],
    last_successful: Option<f64>,
) -> Option<f64> {
    let mut max_used: Option<f64> = None;
    for snap in providers {
        if !snap.is_healthy() {
            continue;
        }
        let Some(p) = snap.headline_percent else {
            continue;
        };
        max_used = Some(match max_used {
            Some(m) => m.max(p),
            None => p,
        });
    }
    max_used.or(last_successful)
}
