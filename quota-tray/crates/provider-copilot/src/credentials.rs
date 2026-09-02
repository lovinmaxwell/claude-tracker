use quota_tray_core::{CredentialError, Credentials};
use secrecy::SecretString;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

pub fn default_apps_json_path() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        dirs::data_local_dir()
            .unwrap_or_default()
            .join("github-copilot/apps.json")
    }
    #[cfg(not(target_os = "windows"))]
    {
        dirs::home_dir()
            .unwrap_or_default()
            .join(".config/github-copilot/apps.json")
    }
}

pub fn parse_apps_json(raw: &str) -> Result<Credentials, CredentialError> {
    let v: Value = serde_json::from_str(raw)
        .map_err(|e| CredentialError::Malformed(e.to_string()))?;
    // Shape: { "github.com:Iv1....": { "oauth_token": "gho_..." }, ... }
    if let Some(obj) = v.as_object() {
        for (_host, entry) in obj {
            if let Some(token) = entry.get("oauth_token").and_then(|t| t.as_str()) {
                if token.starts_with("gho_") || token.starts_with("ghu_") || !token.is_empty() {
                    return Ok(Credentials {
                        raw: SecretString::from(token.to_string()),
                    });
                }
            }
        }
    }
    Err(CredentialError::Missing(
        "no oauth_token in github-copilot apps.json".into(),
    ))
}

pub fn read_apps_json(path: &Path) -> Result<Credentials, CredentialError> {
    let raw = fs::read_to_string(path).map_err(|e| CredentialError::Missing(e.to_string()))?;
    parse_apps_json(&raw)
}

#[cfg(test)]
mod tests {
    use super::*;
    use secrecy::ExposeSecret;

    #[test]
    fn parses_oauth_token_from_apps_json() {
        let raw = r#"{
          "github.com:Iv1.b507a08c87ecfe23": {
            "user": "octo",
            "oauth_token": "gho_testtoken123"
          }
        }"#;
        let creds = parse_apps_json(raw).unwrap();
        assert_eq!(creds.raw.expose_secret(), "gho_testtoken123");
    }
}
