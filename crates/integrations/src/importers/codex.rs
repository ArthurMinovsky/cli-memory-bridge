use std::{fs, path::Path};

use anyhow::{Context, Result};
use cli_memory_core::{
    ProviderKind,
    models::TranscriptMessage,
};
use serde_json::Value;

use super::{ImportedTranscript, message_role};

pub fn import_codex(path: impl AsRef<Path>) -> Result<ImportedTranscript> {
    let path = path.as_ref();
    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read Codex transcript at {}", path.display()))?;

    let mut conversation_id = None;
    let mut messages = Vec::new();

    for raw_line in content.lines() {
        if raw_line.trim().is_empty() {
            continue;
        }

        let line: Value = serde_json::from_str(raw_line)
            .with_context(|| format!("failed to parse Codex JSONL line in {}", path.display()))?;

        match line.get("type").and_then(Value::as_str) {
            Some("session_meta") => {
                conversation_id = line
                    .get("payload")
                    .and_then(|payload| payload.get("id"))
                    .and_then(Value::as_str)
                    .map(str::to_owned)
                    .or(conversation_id);
            }
            Some("response_item") => {
                let payload = line.get("payload").context("Codex response_item missing payload")?;
                if payload.get("type").and_then(Value::as_str) != Some("message") {
                    continue;
                }

                let Some(role) = payload.get("role").and_then(Value::as_str) else {
                    continue;
                };
                let Some(role) = message_role(role) else {
                    continue;
                };

                let text = payload
                    .get("content")
                    .and_then(Value::as_array)
                    .map(|content| extract_codex_text(content))
                    .unwrap_or_default();

                if text.is_empty() {
                    continue;
                }

                messages.push(TranscriptMessage {
                    message_id: format!("msg-{}", messages.len() + 1),
                    role,
                    content: text,
                });
            }
            _ => {}
        }
    }

    Ok(ImportedTranscript {
        provider: ProviderKind::Codex,
        conversation_id: conversation_id.unwrap_or_else(|| path_stem(path)),
        messages,
    })
}

fn extract_codex_text(content: &[Value]) -> String {
    content
        .iter()
        .filter_map(|block| match block.get("type").and_then(Value::as_str) {
            Some("input_text") | Some("output_text") => {
                block.get("text").and_then(Value::as_str).map(str::trim)
            }
            _ => None,
        })
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn path_stem(path: &Path) -> String {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or_default()
        .to_owned()
}
