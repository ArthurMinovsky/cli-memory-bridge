use std::{fs, path::Path};

use anyhow::{Context, Result};
use cli_memory_core::{
    ProviderKind,
    models::{MessageRole, TranscriptMessage},
};
use serde_json::Value;

use super::{ImportedTranscript, path_stem};

pub fn import_hermes(path: impl AsRef<Path>) -> Result<ImportedTranscript> {
    let path = path.as_ref();
    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read Hermes transcript at {}", path.display()))?;

    let mut conversation_id = None;
    let mut messages = Vec::new();

    for raw_line in content.lines() {
        if raw_line.trim().is_empty() {
            continue;
        }

        let line: Value = serde_json::from_str(raw_line)
            .with_context(|| format!("failed to parse Hermes JSONL line in {}", path.display()))?;
        conversation_id = line
            .get("session_id")
            .or_else(|| line.get("sessionId"))
            .and_then(Value::as_str)
            .map(str::to_owned)
            .or(conversation_id);

        let Some(role) = line.get("role").and_then(Value::as_str) else {
            continue;
        };
        let role = match role {
            "user" => MessageRole::User,
            "assistant" => MessageRole::Assistant,
            "system" => MessageRole::System,
            _ => continue,
        };
        let text = line
            .get("content")
            .or_else(|| line.get("text"))
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

    Ok(ImportedTranscript {
        provider: ProviderKind::Hermes,
        conversation_id: conversation_id.unwrap_or_else(|| path_stem(path)),
        messages,
    })
}
