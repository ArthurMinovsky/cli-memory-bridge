use std::{fs, path::Path};

use anyhow::{Context, Result};
use cli_memory_core::{
    ProviderKind,
    models::{MessageRole, TranscriptMessage},
};
use serde_json::Value;

use super::{ImportedTranscript, message_role};

pub fn import_claude(path: impl AsRef<Path>) -> Result<ImportedTranscript> {
    let path = path.as_ref();
    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read Claude transcript at {}", path.display()))?;

    let mut conversation_id = None;
    let mut messages = Vec::new();

    for raw_line in content.lines() {
        if raw_line.trim().is_empty() {
            continue;
        }

        let line: Value = serde_json::from_str(raw_line)
            .with_context(|| format!("failed to parse Claude JSONL line in {}", path.display()))?;

        conversation_id = line
            .get("sessionId")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .or(conversation_id);

        let Some(message) = line.get("message") else {
            continue;
        };
        let default_role = message
            .get("role")
            .and_then(Value::as_str)
            .context("Claude transcript message missing role")?;
        let content = message
            .get("content")
            .context("Claude transcript message missing content")?;

        if let Some(content_blocks) = content.as_array() {
            for (role, text) in extract_claude_messages(content_blocks, default_role)? {
                if text.is_empty() {
                    continue;
                }

                messages.push(TranscriptMessage {
                    message_id: format!("msg-{}", messages.len() + 1),
                    role,
                    content: text,
                });
            }
        } else if let Some(text) = content.as_str().map(str::trim) {
            if let Some(role) = message_role(default_role) {
                if !text.is_empty() {
                    messages.push(TranscriptMessage {
                        message_id: format!("msg-{}", messages.len() + 1),
                        role,
                        content: text.to_owned(),
                    });
                }
            }
        }
    }

    Ok(ImportedTranscript {
        provider: ProviderKind::Claude,
        conversation_id: conversation_id.unwrap_or_else(|| path_stem(path)),
        messages,
    })
}

fn extract_claude_messages(content: &[Value], default_role: &str) -> Result<Vec<(MessageRole, String)>> {
    let mut messages = Vec::new();

    for block in content {
        match block.get("type").and_then(Value::as_str) {
            Some("text") => {
                let Some(role) = message_role(default_role) else {
                    continue;
                };
                let text = block
                    .get("text")
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .unwrap_or_default();
                if !text.is_empty() {
                    messages.push((role, text.to_owned()));
                }
            }
            Some("tool_use") => {
                let name = block.get("name").and_then(Value::as_str).unwrap_or("tool");
                let payload = block
                    .get("input")
                    .map(serde_json::to_string)
                    .transpose()
                    .context("failed to serialize Claude tool input")?
                    .unwrap_or_else(|| "{}".to_owned());
                messages.push((MessageRole::Tool, format!("{name}: {payload}")));
            }
            Some("tool_result") => {
                let text = block
                    .get("content")
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .unwrap_or_default();
                if !text.is_empty() {
                    messages.push((MessageRole::Tool, text.to_owned()));
                }
            }
            _ => {}
        }
    }

    Ok(messages)
}

fn path_stem(path: &Path) -> String {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or_default()
        .to_owned()
}
