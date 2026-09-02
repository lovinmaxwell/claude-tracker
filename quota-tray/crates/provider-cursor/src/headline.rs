use serde_json::Value;

/// Spec formula:
/// totalPercentUsed ?? max(autoPercentUsed, apiPercentUsed) ?? (limit>0 ? 100*totalSpend/limit : None)
pub fn cursor_headline_percent(plan_usage: &Value) -> Option<f64> {
    if let Some(v) = plan_usage.get("totalPercentUsed").and_then(|x| x.as_f64()) {
        return Some(v);
    }
    let auto = plan_usage.get("autoPercentUsed").and_then(|x| x.as_f64());
    let api = plan_usage.get("apiPercentUsed").and_then(|x| x.as_f64());
    match (auto, api) {
        (Some(a), Some(b)) => return Some(a.max(b)),
        (Some(a), None) => return Some(a),
        (None, Some(b)) => return Some(b),
        (None, None) => {}
    }
    let total_spend = plan_usage.get("totalSpend").and_then(|x| x.as_f64());
    let limit = plan_usage.get("limit").and_then(|x| x.as_f64());
    match (total_spend, limit) {
        (Some(spend), Some(lim)) if lim > 0.0 => Some(100.0 * spend / lim),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn prefers_total_percent_used() {
        let v = json!({
            "totalPercentUsed": 41.0,
            "autoPercentUsed": 10.0,
            "apiPercentUsed": 90.0
        });
        assert_eq!(cursor_headline_percent(&v), Some(41.0));
    }

    #[test]
    fn falls_back_to_max_auto_api() {
        let v = json!({
            "autoPercentUsed": 10.0,
            "apiPercentUsed": 55.0
        });
        assert_eq!(cursor_headline_percent(&v), Some(55.0));
    }

    #[test]
    fn falls_back_to_spend_over_limit_cents() {
        let v = json!({ "totalSpend": 250.0, "limit": 1000.0 });
        assert_eq!(cursor_headline_percent(&v), Some(25.0));
    }

    #[test]
    fn none_when_no_signal() {
        let v = json!({});
        assert_eq!(cursor_headline_percent(&v), None);
    }
}
