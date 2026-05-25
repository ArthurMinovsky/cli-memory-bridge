use crate::provider::ProviderKind;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoreConfig {
    pub default_provider: Option<ProviderKind>,
}
