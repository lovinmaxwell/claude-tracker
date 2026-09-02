use serde_json::Value;

fn used_from_bucket(bucket: &Value) -> Option<f64> {
    if bucket.get("unlimited").and_then(|v| v.as_bool()) == Some(true) {
        return None;
    }
    let remaining = bucket.get("percent_remaining")?.as_f64()?;
    Some(100.0 - remaining)
}

/// Prefer premium_interactions; else max(chat, completions); skip unlimited buckets.
pub fn copilot_headline_percent(quota_snapshots: &Value) -> Option<f64> {
    if let Some(premium) = quota_snapshots.get("premium_interactions") {
        if let Some(u) = used_from_bucket(premium) {
            return Some(u);
        }
    }
    let chat = quota_snapshots.get("chat").and_then(used_from_bucket);
    let completions = quota_snapshots
        .get("completions")
        .and_then(used_from_bucket);
    match (chat, completions) {
        (Some(a), Some(b)) => Some(a.max(b)),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn premium_remaining_inverts_to_used() {
        let q = json!({
            "premium_interactions": { "percent_remaining": 40.0, "unlimited": false },
            "chat": { "percent_remaining": 100.0, "unlimited": true }
        });
        assert_eq!(copilot_headline_percent(&q), Some(60.0));
    }

    #[test]
    fn skips_unlimited_premium_and_uses_free_buckets() {
        let q = json!({
            "premium_interactions": { "percent_remaining": 100.0, "unlimited": true },
            "chat": { "percent_remaining": 70.0, "unlimited": false },
            "completions": { "percent_remaining": 50.0, "unlimited": false }
        });
        assert_eq!(copilot_headline_percent(&q), Some(50.0));
    }

    #[test]
    fn none_when_all_unlimited_or_missing() {
        let q = json!({
            "chat": { "unlimited": true },
            "completions": { "unlimited": true }
        });
        assert_eq!(copilot_headline_percent(&q), None);
    }
}
