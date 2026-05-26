use std::{fs, path::Path};

use anyhow::{Context, Result};
use cli_memory_core::{
    ProviderKind,
    models::{MessageRole, TranscriptMessage},
};
use serde_json::Value;

use super::{ImportedTranscript, path_stem};

pub fn import_copilot(path: impl AsRef<Path>) -> Result<ImportedTranscript> {
    let path = path.as_ref();
    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read Copilot transcript at {}", path.display()))?;

    let mut conversation_id = None;
    let mut messages = Vec::new();

    for raw_line in content.lines() {
        if raw_line.trim().is_empty() {
            continue;
        }

        let line: Value = serde_json::from_str(raw_line)
            .with_context(|| format!("failed to parse Copilot JSONL line in {}", path.display()))?;

        match line.get("type").and_then(Value::as_str) {
            Some("session.start") => {
                conversation_id = line
                    .get("data")
                    .and_then(|value| value.get("sessionId"))
                    .and_then(Value::as_str)
                    .map(str::to_owned)
                    .or(conversation_id);
            }
            Some("user") => {
                if let Some(text) = extract_text(&line) {
                    messages.push(TranscriptMessage {
                        message_id: format!("msg-{}", messages.len() + 1),
                        role: MessageRole::User,
                        content: text,
                    });
                }
            }
            Some("user.message") => {
                if let Some(text) = extract_text(&line) {
                    messages.push(TranscriptMessage {
                        message_id: format!("msg-{}", messages.len() + 1),
                        role: MessageRole::User,
                        content: text,
                    });
                }
            }
            Some("assistant") | Some("gemini") | Some("agent.message") => {
                if let Some(text) = extract_text(&line) {
                    messages.push(TranscriptMessage {
                        message_id: format!("msg-{}", messages.len() + 1),
                        role: MessageRole::Assistant,
                        content: text,
                    });
                }
            }
            Some("assistant.message") => {
                if let Some(text) = extract_text(&line) {
                    messages.push(TranscriptMessage {
                        message_id: format!("msg-{}", messages.len() + 1),
                        role: MessageRole::Assistant,
                        content: text,
                    });
                }
            }
            Some("system.message") => {
                if let Some(text) = extract_text(&line) {
                    messages.push(TranscriptMessage {
                        message_id: format!("msg-{}", messages.len() + 1),
                        role: MessageRole::System,
                        content: text,
                    });
                }
            }
            _ => {}
        }
    }

    Ok(ImportedTranscript {
        provider: ProviderKind::Copilot,
        conversation_id: conversation_id.unwrap_or_else(|| path_stem(path)),
        messages,
    })
}

fn extract_text(line: &Value) -> Option<String> {
    if let Some(text) = line
        .get("content")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|text| !text.is_empty())
    {
        return Some(text.to_owned());
    }

    if let Some(text) = line
        .get("data")
        .and_then(|value| value.get("content"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|text| !text.is_empty())
    {
        return Some(text.to_owned());
    }

    if let Some(text) = line
        .get("data")
        .and_then(|value| value.get("message"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|text| !text.is_empty())
    {
        return Some(text.to_owned());
    }

    if let Some(items) = line
        .get("data")
        .and_then(|value| value.get("content"))
        .and_then(Value::as_array)
    {
        let text = items
            .iter()
            .filter_map(|item| item.get("text").and_then(Value::as_str).map(str::trim))
            .filter(|text| !text.is_empty())
            .collect::<Vec<_>>()
            .join("\n\n");
        if !text.is_empty() {
            return Some(text);
        }
    }

    None
}
