//! Cursor credentials (`state.vscdb`) + GetCurrentPeriodUsage fetch.

mod credentials;
mod fetch;
mod headline;

pub use credentials::{default_state_vscdb_path, read_access_token_from_db};
pub use fetch::{parse_usage_body, CursorClient};
pub use headline::cursor_headline_percent;

use quota_tray_core::{
    CredentialError, Credentials, FetchError, Provider, ProviderId, ProviderSnapshot,
};
use std::path::PathBuf;

pub struct CursorProvider {
    pub db_path: PathBuf,
    pub client: CursorClient,
}

impl Default for CursorProvider {
    fn default() -> Self {
        Self {
            db_path: default_state_vscdb_path(),
            client: CursorClient::default(),
        }
    }
}

impl Provider for CursorProvider {
    fn id(&self) -> ProviderId {
        ProviderId::Cursor
    }

    fn credentials(&self) -> Result<Credentials, CredentialError> {
        read_access_token_from_db(&self.db_path)
    }

    fn fetch(&self, creds: &Credentials) -> Result<ProviderSnapshot, FetchError> {
        self.client.fetch_usage(&creds.raw)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_provider_default_id_is_cursor() {
        assert_eq!(CursorProvider::default().id(), ProviderId::Cursor);
    }
}
