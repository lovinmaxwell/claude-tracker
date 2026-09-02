//! Shared types, aggregation, and poller for Quota Tray.

pub fn workspace_smoke() -> &'static str {
    "quota_tray_core"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_smoke_returns_crate_name() {
        assert_eq!(workspace_smoke(), "quota_tray_core");
    }
}
