use std::{fs, path::Path};

use anyhow::{Context, Result};
use cli_memory_core::{
    ProviderKind,
    models::{MessageRole, TranscriptMessage},
};
use serde_json::Value;

use super::{ImportedTranscript, path_stem};

pub fn import_gemini(path: impl AsRef<Path>) -> Result<ImportedTranscript> {
    let path = path.as_ref();
    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read Gemini transcript at {}", path.display()))?;

    let conversation = if path.extension().and_then(|ext| ext.to_str()) == Some("jsonl") {
        import_gemini_jsonl(path, &content)?
    } else {
        import_gemini_json(path, &content)?
    };

    Ok(conversation)
}

fn import_gemini_json(path: &Path, content: &str) -> Result<ImportedTranscript> {
    let root: Value = serde_json::from_str(content)
        .with_context(|| format!("failed to parse Gemini JSON transcript in {}", path.display()))?;

    let conversation_id = root
        .get("sessionId")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .unwrap_or_else(|| path_stem(path));

    let mut messages = Vec::new();

    if let Some(turns) = root.get("turns").and_then(Value::as_array) {
        for turn in turns {
            if let Some(text) = turn
                .get("user")
                .and_then(|user| user.get("text"))
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|text| !text.is_empty())
            {
                messages.push(TranscriptMessage {
                    message_id: format!("msg-{}", messages.len() + 1),
                    role: MessageRole::User,
                    content: text.to_owned(),
                });
            }

            if let Some(text) = turn
                .get("assistant")
                .and_then(|assistant| assistant.get("content"))
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|text| !text.is_empty())
            {
                messages.push(TranscriptMessage {
                    message_id: format!("msg-{}", messages.len() + 1),
                    role: MessageRole::Assistant,
                    content: text.to_owned(),
                });
            }
        }
    }

    if messages.is_empty() {
        if let Some(entries) = root.get("entries").and_then(Value::as_array) {
            for entry in entries {
                if let Some(role) = entry.get("role").and_then(Value::as_str) {
                    let role = match role {
                        "user" => Some(MessageRole::User),
                        "assistant" | "gemini" => Some(MessageRole::Assistant),
                        "system" => Some(MessageRole::System),
                        _ => None,
                    };
                    let Some(role) = role else {
                        continue;
                    };

                    if let Some(text) = entry
                        .get("content")
                        .or_else(|| entry.get("text"))
                        .and_then(Value::as_str)
                        .map(str::trim)
                        .filter(|text| !text.is_empty())
                    {
                        messages.push(TranscriptMessage {
                            message_id: format!("msg-{}", messages.len() + 1),
                            role,
                            content: text.to_owned(),
                        });
                    }
                }
            }
        }
    }

    Ok(ImportedTranscript {
        provider: ProviderKind::Gemini,
        conversation_id,
        messages,
    })
}

fn import_gemini_jsonl(path: &Path, content: &str) -> Result<ImportedTranscript> {
    let mut conversation_id = None;
    let mut messages = Vec::new();

    for raw_line in content.lines() {
        if raw_line.trim().is_empty() {
            continue;
        }

        let line: Value = serde_json::from_str(raw_line)
            .with_context(|| format!("failed to parse Gemini JSONL line in {}", path.display()))?;

        conversation_id = line
            .get("sessionId")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .or(conversation_id);

        let Some(kind) = line.get("type").and_then(Value::as_str) else {
            continue;
        };

        let role = match kind {
            "user" => MessageRole::User,
            "gemini" | "assistant" => MessageRole::Assistant,
            "system" | "info" => MessageRole::System,
            _ => continue,
        };

        let text = extract_text(&line).unwrap_or_default();
        if text.is_empty() {
            continue;
        }

        messages.push(TranscriptMessage {
            message_id: format!("msg-{}", messages.len() + 1),
            role,
            content: text,
        });
    }

    Ok(ImportedTranscript {
        provider: ProviderKind::Gemini,
        conversation_id: conversation_id.unwrap_or_else(|| path_stem(path)),
        messages,
    })
}

fn extract_text(value: &Value) -> Option<String> {
    if let Some(text) = value
        .get("content")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|text| !text.is_empty())
    {
        return Some(text.to_owned());
    }

    if let Some(items) = value.get("content").and_then(Value::as_array) {
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

    value.get("text")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_owned)
}
