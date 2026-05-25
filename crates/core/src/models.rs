use crate::{ids::derive_resume_hash, provider::ProviderKind};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversationLocator {
    pub provider: ProviderKind,
    pub conversation_id: String,
}

impl ConversationLocator {
    pub fn resume_hash(&self) -> String {
        derive_resume_hash(self.provider, &self.conversation_id)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversationTranscript {
    pub locator: ConversationLocator,
    pub messages: Vec<TranscriptMessage>,
}

impl ConversationTranscript {
    pub fn resume_hash(&self) -> String {
        self.locator.resume_hash()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TranscriptMessage {
    pub message_id: String,
    pub role: MessageRole,
    pub content: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}
