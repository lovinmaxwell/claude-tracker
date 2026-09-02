//! Claude Code credentials + Anthropic OAuth usage fetch.

mod credentials;
mod provider;
mod usage;

pub use credentials::{
    credentials_file_path, parse_claude_credentials_json, ClaudeOAuthCreds, KEYCHAIN_SERVICE,
};
pub use provider::ClaudeProvider;
pub use usage::{
    fetch_claude_usage, ANTHROPIC_BETA_OAUTH, DEFAULT_API_BASE, USAGE_PATH,
};

pub fn provider_id_str() -> &'static str {
    "claude"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_id_str_is_claude() {
        assert_eq!(provider_id_str(), "claude");
    }

    #[test]
    fn claude_provider_default_id_is_claude() {
        use quota_tray_core::{Provider, ProviderId};
        assert_eq!(ClaudeProvider::default().id(), ProviderId::Claude);
    }
}
