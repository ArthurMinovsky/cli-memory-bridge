pub mod antigravity_cli;
pub mod claude;
pub mod copilot;
pub mod codex;
pub mod gemini;
pub mod hermes;
pub mod opencode;
pub mod zed;

use cli_memory_core::{
    ProviderKind,
    models::{ConversationLocator, ConversationTranscript, MessageRole, TranscriptMessage},
};

pub use antigravity_cli::import_antigravity_cli;
pub use claude::import_claude;
pub use copilot::import_copilot;
pub use codex::import_codex;
pub use gemini::import_gemini;
pub use hermes::import_hermes;
pub use opencode::import_opencode;
pub use zed::import_zed;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImportedTranscript {
    pub provider: ProviderKind,
    pub conversation_id: String,
    pub messages: Vec<TranscriptMessage>,
}

impl ImportedTranscript {
    pub fn into_conversation(self) -> ConversationTranscript {
        ConversationTranscript {
            locator: ConversationLocator {
                provider: self.provider,
                conversation_id: self.conversation_id,
            },
            messages: self.messages,
        }
    }
}

fn message_role(role: &str) -> Option<MessageRole> {
    match role {
        "user" => Some(MessageRole::User),
        "assistant" => Some(MessageRole::Assistant),
        "tool" => Some(MessageRole::Tool),
        "system" => Some(MessageRole::System),
        _ => None,
    }
}

pub fn path_stem(path: &std::path::Path) -> String {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or_default()
        .to_owned()
}
