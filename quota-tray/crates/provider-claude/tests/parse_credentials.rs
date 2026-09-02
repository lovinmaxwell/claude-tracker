use provider_claude::{parse_claude_credentials_json, KEYCHAIN_SERVICE};
use quota_tray_core::CredentialError;

/// Mirrors claude-tracker `tests/Pest.php` `claudeKeychainBlob` shape
/// (and the locked sample from the Quota Tray plan).
const SAMPLE: &str = r#"{"claudeAiOauth":{"accessToken":"sk-ant-oat-test","expiresAt":9999999999999}}"#;

const FULL_BLOB: &str = r#"{
  "claudeAiOauth": {
    "accessToken": "sk-ant-oat01-testing",
    "refreshToken": "sk-ant-ort01-testing",
    "expiresAt": 9999999999999,
    "subscriptionType": "pro"
  },
  "organizationUuid": "org-testing"
}"#;

#[test]
fn parses_locked_sample_blob() {
    let creds = parse_claude_credentials_json(SAMPLE).expect("parse sample");
    assert_eq!(creds.access_token, "sk-ant-oat-test");
    assert_eq!(creds.expires_at_ms, Some(9999999999999));
}

#[test]
fn parses_full_tracker_style_blob() {
    let creds = parse_claude_credentials_json(FULL_BLOB).expect("parse full");
    assert_eq!(creds.access_token, "sk-ant-oat01-testing");
    assert_eq!(creds.expires_at_ms, Some(9999999999999));
}

#[test]
fn rejects_missing_oauth_object() {
    let err = parse_claude_credentials_json("{}").unwrap_err();
    assert!(matches!(err, CredentialError::Malformed(_)));
}

#[test]
fn rejects_empty_access_token() {
    let json = r#"{"claudeAiOauth":{"accessToken":"","expiresAt":1}}"#;
    let err = parse_claude_credentials_json(json).unwrap_err();
    assert!(matches!(err, CredentialError::Malformed(_)));
}

#[test]
fn keychain_service_name_matches_claude_code() {
    assert_eq!(KEYCHAIN_SERVICE, "Claude Code-credentials");
}
