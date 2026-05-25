use chrono::{DateTime, Utc};
use cli_memory_core::ProviderKind;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Checkpoint {
    pub provider: ProviderKind,
    pub source_path: String,
    pub fingerprint: String,
    pub updated_at: DateTime<Utc>,
}
