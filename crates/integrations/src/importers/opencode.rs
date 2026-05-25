use std::{fs, path::{Path, PathBuf}};

use anyhow::{Context, Result};
use cli_memory_core::{
    ProviderKind,
    models::{MessageRole, TranscriptMessage},
};
use rusqlite::{Connection, params};
use serde_json::Value;

use super::{ImportedTranscript, path_stem};

pub fn import_opencode(path: impl AsRef<Path>) -> Result<ImportedTranscript> {
    let path = path.as_ref();
    if path.file_name().and_then(|name| name.to_str()) == Some("opencode.db") {
        return import_latest_session_from_db(path);
    }

    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read OpenCode session at {}", path.display()))?;
    let root: Value = serde_json::from_str(&content)
        .with_context(|| format!("failed to parse OpenCode JSON in {}", path.display()))?;

    if root.as_array().is_some_and(|items| items.is_empty()) {
        if let Some(db_path) = find_opencode_db(path) {
            let session_id = path_stem(path);
            return import_session_from_db(&db_path, &session_id);
        }
    }

    let conversation_id = root
        .get("session_id")
        .or_else(|| root.get("sessionId"))
        .or_else(|| root.get("id"))
        .and_then(Value::as_str)
        .map(str::to_owned)
        .unwrap_or_else(|| path_stem(path));

    let mut messages = Vec::new();
    if let Some(entries) = root.get("messages").and_then(Value::as_array) {
        for entry in entries {
            let Some(role) = entry.get("role").and_then(Value::as_str) else {
                continue;
            };
            let role = match role {
                "user" => MessageRole::User,
                "assistant" => MessageRole::Assistant,
                "system" => MessageRole::System,
                _ => continue,
            };
            let text = entry
                .get("content")
                .or_else(|| entry.get("text"))
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

    Ok(ImportedTranscript {
        provider: ProviderKind::OpenCode,
        conversation_id,
        messages,
    })
}

fn import_latest_session_from_db(db_path: &Path) -> Result<ImportedTranscript> {
    let connection = Connection::open(db_path)
        .with_context(|| format!("failed to open OpenCode database {}", db_path.display()))?;
    let session_id: String = connection
        .query_row(
            "SELECT id FROM session ORDER BY time_created DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .context("failed to load latest OpenCode session id")?;
    import_session_with_connection(&connection, &session_id)
}

fn import_session_from_db(db_path: &Path, session_id: &str) -> Result<ImportedTranscript> {
    let connection = Connection::open(db_path)
        .with_context(|| format!("failed to open OpenCode database {}", db_path.display()))?;
    import_session_with_connection(&connection, session_id)
}

fn import_session_with_connection(
    connection: &Connection,
    session_id: &str,
) -> Result<ImportedTranscript> {
    let mut statement = connection
        .prepare("SELECT id, data FROM message WHERE session_id = ?1 ORDER BY time_created ASC, id ASC")
        .context("failed to prepare OpenCode message query")?;
    let rows = statement
        .query_map(params![session_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .context("failed to query OpenCode messages")?;

    let mut messages = Vec::new();
    for row in rows {
        let (message_id, data) = row.context("failed to read OpenCode message row")?;
        let message_value: Value = serde_json::from_str(&data)
            .context("failed to parse OpenCode message data JSON")?;
        let Some(role) = message_value.get("role").and_then(Value::as_str) else {
            continue;
        };
        let role = match role {
            "user" => MessageRole::User,
            "assistant" => MessageRole::Assistant,
            "system" => MessageRole::System,
            _ => continue,
        };

        let mut part_statement = connection
            .prepare("SELECT data FROM part WHERE message_id = ?1 ORDER BY time_created ASC, id ASC")
            .context("failed to prepare OpenCode part query")?;
        let part_rows = part_statement
            .query_map(params![&message_id], |row| row.get::<_, String>(0))
            .context("failed to query OpenCode parts")?;

        let mut chunks = Vec::new();
        for part in part_rows {
            let part = part.context("failed to read OpenCode part row")?;
            let value: Value = serde_json::from_str(&part)
                .context("failed to parse OpenCode part data JSON")?;
            if value.get("type").and_then(Value::as_str) != Some("text") {
                continue;
            }
            let text = value
                .get("text")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|text| !text.is_empty())
                .unwrap_or_default();
            if !text.is_empty() {
                chunks.push(text.to_owned());
            }
        }

        if chunks.is_empty() {
            continue;
        }

        messages.push(TranscriptMessage {
            message_id,
            role,
            content: chunks.join("\n\n"),
        });
    }

    Ok(ImportedTranscript {
        provider: ProviderKind::OpenCode,
        conversation_id: session_id.to_owned(),
        messages,
    })
}

fn find_opencode_db(path: &Path) -> Option<PathBuf> {
    path.ancestors()
        .map(|ancestor| ancestor.join("opencode.db"))
        .find(|candidate| candidate.exists())
}
