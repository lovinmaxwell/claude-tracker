use dirs::home_dir;
use quota_tray_core::CredentialError;
use serde::Deserialize;
use std::path::PathBuf;

/// macOS Keychain generic-password service name (Claude Code).
pub const KEYCHAIN_SERVICE: &str = "Claude Code-credentials";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaudeOAuthCreds {
    pub access_token: String,
    pub expires_at_ms: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct Root {
    #[serde(rename = "claudeAiOauth")]
    claude_ai_oauth: Option<OAuthBlock>,
}

#[derive(Debug, Deserialize)]
struct OAuthBlock {
    #[serde(rename = "accessToken")]
    access_token: Option<String>,
    #[serde(rename = "expiresAt")]
    expires_at: Option<i64>,
}

/// Parse the JSON blob Claude Code stores in Keychain / `.credentials.json`.
/// Does not read Keychain or the filesystem — callers pass the blob.
pub fn parse_claude_credentials_json(json: &str) -> Result<ClaudeOAuthCreds, CredentialError> {
    let root: Root = serde_json::from_str(json)
        .map_err(|e| CredentialError::Malformed(e.to_string()))?;
    let oauth = root
        .claude_ai_oauth
        .ok_or_else(|| CredentialError::Malformed("missing claudeAiOauth".into()))?;
    let token = oauth
        .access_token
        .filter(|t| !t.is_empty())
        .ok_or_else(|| CredentialError::Malformed("missing accessToken".into()))?;
    Ok(ClaudeOAuthCreds {
        access_token: token,
        expires_at_ms: oauth.expires_at,
    })
}

/// Linux/Win (and macOS fallback) path: `$CLAUDE_CONFIG_DIR/.credentials.json`
/// or `~/.claude/.credentials.json`.
pub fn credentials_file_path() -> PathBuf {
    if let Ok(dir) = std::env::var("CLAUDE_CONFIG_DIR") {
        return PathBuf::from(dir).join(".credentials.json");
    }
    home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude")
        .join(".credentials.json")
}
