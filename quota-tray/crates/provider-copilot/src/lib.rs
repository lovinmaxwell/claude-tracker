//! GitHub Copilot credentials (`apps.json`) + `copilot_internal/user` fetch.

mod credentials;
mod fetch;
mod headline;

pub use credentials::{default_apps_json_path, parse_apps_json, read_apps_json};
pub use fetch::{parse_copilot_user, CopilotClient};
pub use headline::copilot_headline_percent;

use quota_tray_core::{
    CredentialError, Credentials, FetchError, Provider, ProviderId, ProviderSnapshot,
};
use std::path::PathBuf;

pub struct CopilotProvider {
    pub apps_json_path: PathBuf,
    pub client: CopilotClient,
}

impl Default for CopilotProvider {
    fn default() -> Self {
        Self {
            apps_json_path: default_apps_json_path(),
            client: CopilotClient::default(),
        }
    }
}

impl Provider for CopilotProvider {
    fn id(&self) -> ProviderId {
        ProviderId::Copilot
    }

    fn credentials(&self) -> Result<Credentials, CredentialError> {
        read_apps_json(&self.apps_json_path)
    }

    fn fetch(&self, creds: &Credentials) -> Result<ProviderSnapshot, FetchError> {
        self.client.fetch_user(&creds.raw)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copilot_provider_default_id_is_copilot() {
        assert_eq!(CopilotProvider::default().id(), ProviderId::Copilot);
    }
}
