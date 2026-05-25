use std::{fs, path::Path};

use anyhow::{Context, Result};
use cli_memory_core::{
    ProviderKind,
    models::{MessageRole, TranscriptMessage},
};
use serde_json::Value;

use super::{ImportedTranscript, path_stem};

pub fn import_zed(path: impl AsRef<Path>) -> Result<ImportedTranscript> {
    let path = path.as_ref();
    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read Zed conversation at {}", path.display()))?;
    let root: Value = serde_json::from_str(&content)
        .with_context(|| format!("failed to parse Zed conversation JSON in {}", path.display()))?;

    let conversation_id = root
        .get("id")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .unwrap_or_else(|| path_stem(path));

    let summary = root
        .get("summary")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .unwrap_or_default();

    let mut messages = Vec::new();
    if let Some(entries) = root.get("messages").and_then(Value::as_array) {
        for entry in entries {
            let Some(role) = entry
                .get("metadata")
                .and_then(|value| value.get("role"))
                .and_then(Value::as_str)
            else {
                continue;
            };
            let role = match role {
                "user" => MessageRole::User,
                "assistant" => MessageRole::Assistant,
                "system" => MessageRole::System,
                _ => continue,
            };

            let text = entry
                .get("text")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|text| !text.is_empty())
                .unwrap_or_default();

            if text.is_empty() {
                continue;
            }

            messages.push(TranscriptMessage {
                message_id: format!("msg-{}", messages.len() + 1),
                role,
                content: text.to_owned(),
            });
        }
    }

    let has_substantive_text = messages
        .iter()
        .any(|message| matches!(message.role, MessageRole::User | MessageRole::Assistant));

    if has_substantive_text && !summary.is_empty() {
        messages.insert(
            0,
            TranscriptMessage {
                message_id: "msg-1".to_owned(),
                role: MessageRole::System,
                content: format!("Summary: {summary}"),
            },
        );
        for (index, message) in messages.iter_mut().enumerate().skip(1) {
            message.message_id = format!("msg-{}", index + 1);
        }
    }

    Ok(ImportedTranscript {
        provider: ProviderKind::Zed,
        conversation_id,
        messages,
    })
}
