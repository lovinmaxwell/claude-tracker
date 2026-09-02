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

pub fn default_hosts_json_path() -> PathBuf {
    default_apps_json_path().with_file_name("hosts.json")
}

/// Shared shape for Copilot `apps.json` and legacy `hosts.json`.
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
        "no oauth_token in github-copilot credential json".into(),
    ))
}

pub fn parse_hosts_json(raw: &str) -> Result<Credentials, CredentialError> {
    parse_apps_json(raw)
}

pub fn read_apps_json(path: &Path) -> Result<Credentials, CredentialError> {
    let raw = fs::read_to_string(path).map_err(|e| CredentialError::Missing(e.to_string()))?;
    parse_apps_json(&raw)
}

/// Prefer `apps.json`; fall back to sibling / default `hosts.json` (legacy installs).
pub fn read_copilot_credentials(apps_path: &Path) -> Result<Credentials, CredentialError> {
    match read_apps_json(apps_path) {
        Ok(creds) => Ok(creds),
        Err(apps_err) => {
            let sibling_hosts = apps_path.with_file_name("hosts.json");
            if let Ok(creds) = read_apps_json(&sibling_hosts) {
                return Ok(creds);
            }
            let default_hosts = default_hosts_json_path();
            if default_hosts != sibling_hosts {
                if let Ok(creds) = read_apps_json(&default_hosts) {
                    return Ok(creds);
                }
            }
            Err(apps_err)
        }
    }
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

    #[test]
    fn parses_oauth_token_from_hosts_json_fixture() {
        let raw = r#"{
          "github.com": {
            "user": "legacy",
            "oauth_token": "gho_hosts_legacy_token"
          }
        }"#;
        let creds = parse_hosts_json(raw).unwrap();
        assert_eq!(creds.raw.expose_secret(), "gho_hosts_legacy_token");
    }

    #[test]
    fn falls_back_to_hosts_json_when_apps_missing() {
        let dir = tempfile::tempdir().unwrap();
        let apps = dir.path().join("apps.json");
        let hosts = dir.path().join("hosts.json");
        fs::write(
            &hosts,
            r#"{"github.com":{"oauth_token":"gho_from_hosts"}}"#,
        )
        .unwrap();
        let creds = read_copilot_credentials(&apps).unwrap();
        assert_eq!(creds.raw.expose_secret(), "gho_from_hosts");
    }

    #[test]
    fn prefers_apps_json_over_hosts() {
        let dir = tempfile::tempdir().unwrap();
        let apps = dir.path().join("apps.json");
        let hosts = dir.path().join("hosts.json");
        fs::write(
            &apps,
            r#"{"github.com:Iv1.x":{"oauth_token":"gho_from_apps"}}"#,
        )
        .unwrap();
        fs::write(
            &hosts,
            r#"{"github.com":{"oauth_token":"gho_from_hosts"}}"#,
        )
        .unwrap();
        let creds = read_copilot_credentials(&apps).unwrap();
        assert_eq!(creds.raw.expose_secret(), "gho_from_apps");
    }
}
