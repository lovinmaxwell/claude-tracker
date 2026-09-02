use crate::credentials::{
    credentials_file_path, parse_claude_credentials_json, KEYCHAIN_SERVICE,
};
use crate::usage::{fetch_claude_usage, DEFAULT_API_BASE};
use quota_tray_core::{
    ClaudeHeadlineMetric, CredentialError, Credentials, FetchError, Provider, ProviderId,
    ProviderSnapshot,
};
use secrecy::{ExposeSecret, SecretString};
use std::process::Command;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

static CLAUDE_RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

fn claude_runtime() -> &'static tokio::runtime::Runtime {
    CLAUDE_RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("claude provider tokio runtime")
    })
}

/// Claude Code usage provider: Keychain/file creds + Anthropic OAuth usage.
#[derive(Clone, Debug)]
pub struct ClaudeProvider {
    pub headline: ClaudeHeadlineMetric,
    pub api_base: String,
}

impl Default for ClaudeProvider {
    fn default() -> Self {
        Self {
            headline: ClaudeHeadlineMetric::Highest,
            api_base: DEFAULT_API_BASE.to_string(),
        }
    }
}

impl Provider for ClaudeProvider {
    fn id(&self) -> ProviderId {
        ProviderId::Claude
    }

    fn credentials(&self) -> Result<Credentials, CredentialError> {
        let json = read_credentials_blob()?;
        let oauth = parse_claude_credentials_json(&json)?;
        if token_expired(oauth.expires_at_ms) {
            return Err(CredentialError::Expired);
        }
        Ok(Credentials {
            raw: SecretString::from(oauth.access_token),
        })
    }

    fn fetch(&self, creds: &Credentials) -> Result<ProviderSnapshot, FetchError> {
        let token = creds.raw.expose_secret().to_string();
        let base = self.api_base.clone();
        let headline = self.headline.clone();
        claude_runtime().block_on(fetch_claude_usage(&base, &token, headline))
    }
}

fn token_expired(expires_at_ms: Option<i64>) -> bool {
    let Some(expires_at_ms) = expires_at_ms else {
        return false;
    };
    let now_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0);
    (expires_at_ms as f64 / 1000.0) <= now_secs
}

fn read_credentials_blob() -> Result<String, CredentialError> {
    #[cfg(target_os = "macos")]
    {
        match read_keychain_blob() {
            Ok(blob) => return Ok(blob),
            Err(CredentialError::Missing(_)) => {}
            Err(other) => return Err(other),
        }
    }

    let path = credentials_file_path();
    std::fs::read_to_string(&path)
        .map_err(|e| CredentialError::Missing(format!("{}: {e}", path.display())))
}

#[cfg(target_os = "macos")]
fn read_keychain_blob() -> Result<String, CredentialError> {
    let output = Command::new("/usr/bin/security")
        .args(["find-generic-password", "-s", KEYCHAIN_SERVICE, "-w"])
        .output()
        .map_err(|e| CredentialError::Missing(e.to_string()))?;

    if output.status.code() == Some(44) {
        return Err(CredentialError::Missing(format!(
            "no Keychain item for {KEYCHAIN_SERVICE}"
        )));
    }

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        let detail = err.trim();
        return Err(CredentialError::Missing(if detail.is_empty() {
            "Keychain access denied".into()
        } else {
            detail.to_string()
        }));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
