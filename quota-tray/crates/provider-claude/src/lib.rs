//! Claude Code credentials + Anthropic OAuth usage fetch.

mod credentials;

pub use credentials::{
    credentials_file_path, parse_claude_credentials_json, ClaudeOAuthCreds, KEYCHAIN_SERVICE,
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
}
