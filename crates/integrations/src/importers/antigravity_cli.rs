use std::{fs, path::Path};

use anyhow::{Context, Result};
use cli_memory_core::{
    ProviderKind,
    models::{MessageRole, TranscriptMessage},
};
use serde_json::Value;

use super::ImportedTranscript;

pub fn import_antigravity_cli(path: impl AsRef<Path>) -> Result<ImportedTranscript> {
    let path = path.as_ref();
    let conversation_id = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_owned();

    let mut messages = Vec::new();
    let mut seen_texts = std::collections::BTreeSet::new();

    for name in [
        "task.md",
        "implementation_plan.md",
        "walkthrough.md",
        "offline_guide.md",
        "output_format_guide.md",
        "best_examples_per_label.md",
    ] {
        let candidate = path.join(name);
        if !candidate.exists() {
            continue;
        }

        push_artifact_message(&candidate, &mut messages, &mut seen_texts)?;
        push_artifact_summary(&candidate, &mut messages, &mut seen_texts)?;
    }

    Ok(ImportedTranscript {
        provider: ProviderKind::AntigravityCli,
        conversation_id,
        messages,
    })
}

fn push_artifact_message(
    candidate: &Path,
    messages: &mut Vec<TranscriptMessage>,
    seen_texts: &mut std::collections::BTreeSet<String>,
) -> Result<()> {
    let content = fs::read_to_string(candidate).with_context(|| {
        format!(
            "failed to read Antigravity CLI artifact at {}",
            candidate.display()
        )
    })?;
    let text = content.trim();
    if text.is_empty() || !seen_texts.insert(text.to_owned()) {
        return Ok(());
    }

    let role = if candidate.file_name().and_then(|name| name.to_str()) == Some("task.md") {
        MessageRole::User
    } else {
        MessageRole::Assistant
    };
    messages.push(TranscriptMessage {
        message_id: format!("msg-{}", messages.len() + 1),
        role,
        content: text.to_owned(),
    });

    Ok(())
}

fn push_artifact_summary(
    candidate: &Path,
    messages: &mut Vec<TranscriptMessage>,
    seen_texts: &mut std::collections::BTreeSet<String>,
) -> Result<()> {
    let metadata = candidate.with_extension(format!(
        "{}.metadata.json",
        candidate
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or_default()
    ));
    if !metadata.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(&metadata).with_context(|| {
        format!(
            "failed to read Antigravity CLI metadata at {}",
            metadata.display()
        )
    })?;
    let value: Value = serde_json::from_str(&content).with_context(|| {
        format!(
            "failed to parse Antigravity CLI metadata at {}",
            metadata.display()
        )
    })?;
    let Some(summary) = value
        .get("summary")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|text| !text.is_empty())
    else {
        return Ok(());
    };

    let artifact_type = value
        .get("artifactType")
        .and_then(Value::as_str)
        .unwrap_or("ARTIFACT_TYPE_OTHER");
    let text = format!("Summary ({artifact_type}): {summary}");
    if !seen_texts.insert(text.clone()) {
        return Ok(());
    }

    messages.push(TranscriptMessage {
        message_id: format!("msg-{}", messages.len() + 1),
        role: MessageRole::Assistant,
        content: text,
    });
    Ok(())
}
