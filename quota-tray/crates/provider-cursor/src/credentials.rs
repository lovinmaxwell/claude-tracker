use quota_tray_core::{CredentialError, Credentials};
use rusqlite::Connection;
use secrecy::SecretString;
use std::path::{Path, PathBuf};

pub fn default_state_vscdb_path() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        dirs::home_dir()
            .unwrap_or_default()
            .join("Library/Application Support/Cursor/User/globalStorage/state.vscdb")
    }
    #[cfg(target_os = "linux")]
    {
        dirs::home_dir()
            .unwrap_or_default()
            .join(".config/Cursor/User/globalStorage/state.vscdb")
    }
    #[cfg(target_os = "windows")]
    {
        dirs::data_dir()
            .unwrap_or_default()
            .join("Cursor/User/globalStorage/state.vscdb")
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        dirs::home_dir()
            .unwrap_or_default()
            .join(".config/Cursor/User/globalStorage/state.vscdb")
    }
}

/// Read `cursorAuth/accessToken` from Cursor's `state.vscdb` (immutable / read-only).
pub fn read_access_token_from_db(path: &Path) -> Result<Credentials, CredentialError> {
    // Immutable / read-only to coexist with Cursor's WAL lock.
    let uri = format!("file:{}?mode=ro", path.display());
    let conn = Connection::open_with_flags(
        &uri,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
    )
    .map_err(|e| CredentialError::Missing(e.to_string()))?;

    let token: String = conn
        .query_row(
            "SELECT value FROM ItemTable WHERE key = ?1",
            ["cursorAuth/accessToken"],
            |row| row.get(0),
        )
        .map_err(|e| CredentialError::Missing(e.to_string()))?;

    if token.trim().is_empty() {
        return Err(CredentialError::Missing(
            "cursorAuth/accessToken empty".into(),
        ));
    }
    Ok(Credentials {
        raw: SecretString::from(token),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use secrecy::ExposeSecret;

    #[test]
    fn reads_token_from_itemtable() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.vscdb");
        {
            let conn = Connection::open(&path).unwrap();
            conn.execute_batch(
                "CREATE TABLE ItemTable (key TEXT PRIMARY KEY, value TEXT);
                 INSERT INTO ItemTable(key, value) VALUES ('cursorAuth/accessToken', 'jwt-test-token');",
            )
            .unwrap();
        }
        let creds = read_access_token_from_db(&path).unwrap();
        assert_eq!(creds.raw.expose_secret(), "jwt-test-token");
    }

    #[test]
    fn empty_token_is_missing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.vscdb");
        {
            let conn = Connection::open(&path).unwrap();
            conn.execute_batch(
                "CREATE TABLE ItemTable (key TEXT PRIMARY KEY, value TEXT);
                 INSERT INTO ItemTable(key, value) VALUES ('cursorAuth/accessToken', '   ');",
            )
            .unwrap();
        }
        match read_access_token_from_db(&path) {
            Err(CredentialError::Missing(_)) => {}
            Err(other) => panic!("expected Missing, got {other}"),
            Ok(_) => panic!("expected Missing, got Ok"),
        }
    }
}
