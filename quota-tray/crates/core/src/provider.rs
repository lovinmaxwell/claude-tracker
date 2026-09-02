use crate::types::{ProviderId, ProviderSnapshot};
use secrecy::SecretString;
use thiserror::Error;

/// Opaque per-provider secret material; never logged.
pub struct Credentials {
    pub raw: SecretString,
}

#[derive(Debug, Error)]
pub enum CredentialError {
    #[error("credentials missing: {0}")]
    Missing(String),
    #[error("credentials malformed: {0}")]
    Malformed(String),
    #[error("credentials expired")]
    Expired,
}

#[derive(Debug, Error)]
pub enum FetchError {
    #[error("network: {0}")]
    Network(String),
    #[error("http {status}: {body}")]
    Http { status: u16, body: String },
    #[error("parse: {0}")]
    Parse(String),
}

pub trait Provider: Send + Sync {
    fn id(&self) -> ProviderId;
    fn credentials(&self) -> Result<Credentials, CredentialError>;
    fn fetch(&self, creds: &Credentials) -> Result<ProviderSnapshot, FetchError>;
}
