use std::{io, path::Path};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use cli_memory_core::{
    ProviderKind,
    models::{ConversationTranscript, MessageRole},
};
use rusqlite::{Connection, params, types::Type};
use serde_json::Value;

use crate::checkpoints::Checkpoint;

pub struct Storage {
    connection: Connection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckpointRow {
    pub id: i64,
    pub provider: ProviderKind,
    pub source_path: String,
    pub fingerprint: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MessageRow {
    pub provider: ProviderKind,
    pub conversation_id: String,
    pub message_id: String,
    pub role: MessageRole,
    pub content: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EmbeddedMessageRow {
    pub provider: ProviderKind,
    pub conversation_id: String,
    pub message_id: String,
    pub role: MessageRole,
    pub content: String,
    pub embedding: Vec<f32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConversationStateRow {
    pub provider: ProviderKind,
    pub conversation_id: String,
    pub source_path: String,
    pub source_fingerprint: String,
    pub imported_at: DateTime<Utc>,
    pub forgotten_at: Option<DateTime<Utc>>,
    pub ban_reason: Option<String>,
}

impl Storage {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let connection = Connection::open(path).context("failed to open SQLite storage")?;
        let storage = Self { connection };
        storage.initialize()?;
        Ok(storage)
    }

    pub fn initialize(&self) -> Result<()> {
        self.connection
            .execute_batch(include_str!("schema.sql"))
            .context("failed to initialize SQLite schema")
    }

    pub fn save_checkpoint(&self, checkpoint: &Checkpoint) -> Result<()> {
        self.connection
            .execute(
                "INSERT INTO checkpoints (provider, source_path, fingerprint, updated_at)
                 VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(provider, source_path) DO UPDATE SET
                     fingerprint = excluded.fingerprint,
                     updated_at = excluded.updated_at",
                params![
                    checkpoint.provider.as_slug(),
                    &checkpoint.source_path,
                    &checkpoint.fingerprint,
                    checkpoint.updated_at.to_rfc3339(),
                ],
            )
            .context("failed to save checkpoint")?;

        Ok(())
    }

    pub fn list_checkpoints(&self) -> Result<Vec<CheckpointRow>> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT id, provider, source_path, fingerprint, updated_at
                 FROM checkpoints
                 ORDER BY provider ASC, source_path ASC",
            )
            .context("failed to prepare checkpoint listing query")?;

        let rows = statement
            .query_map([], |row| {
                let provider_slug: String = row.get(1)?;
                let provider = ProviderKind::from_slug(&provider_slug).map_err(|error| {
                    rusqlite::Error::FromSqlConversionFailure(1, Type::Text, Box::new(error))
                })?;

                let updated_at = parse_updated_at(row.get::<_, String>(4)?)?;

                Ok(CheckpointRow {
                    id: row.get(0)?,
                    provider,
                    source_path: row.get(2)?,
                    fingerprint: row.get(3)?,
                    updated_at,
                })
            })
            .context("failed to query checkpoints")?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .context("failed to read checkpoint rows")
    }

    pub fn save_transcript(&self, transcript: &ConversationTranscript) -> Result<()> {
        let provider = transcript.locator.provider.as_slug();
        let conversation_id = &transcript.locator.conversation_id;
        let resume_hash = transcript.resume_hash();
        let imported_at = Utc::now().to_rfc3339();

        self.connection
            .execute(
                "INSERT INTO conversations (provider, conversation_id, resume_hash, imported_at)
                 VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(provider, conversation_id) DO UPDATE SET
                     resume_hash = excluded.resume_hash,
                     imported_at = excluded.imported_at",
                params![provider, conversation_id, resume_hash, imported_at],
            )
            .context("failed to save conversation")?;

        for message in &transcript.messages {
            self.connection
                .execute(
                    "INSERT INTO messages (provider, conversation_id, message_id, role, content)
                     VALUES (?1, ?2, ?3, ?4, ?5)
                     ON CONFLICT(provider, conversation_id, message_id) DO UPDATE SET
                         role = excluded.role,
                         content = excluded.content",
                    params![
                        provider,
                        conversation_id,
                        &message.message_id,
                        message_role_slug(message.role),
                        &message.content,
                    ],
                )
                .with_context(|| {
                    format!(
                        "failed to save message {} for {}:{}",
                        message.message_id, provider, conversation_id
                    )
                })?;
        }

        Ok(())
    }

    pub fn save_conversation_state(
        &self,
        provider: ProviderKind,
        conversation_id: &str,
        source_path: &str,
        source_fingerprint: &str,
    ) -> Result<()> {
        self.connection
            .execute(
                "INSERT INTO conversation_states
                    (provider, conversation_id, source_path, source_fingerprint, imported_at, forgotten_at, ban_reason)
                 VALUES (?1, ?2, ?3, ?4, ?5, NULL, NULL)
                 ON CONFLICT(provider, conversation_id) DO UPDATE SET
                    source_path = excluded.source_path,
                    source_fingerprint = excluded.source_fingerprint,
                    imported_at = excluded.imported_at",
                params![
                    provider.as_slug(),
                    conversation_id,
                    source_path,
                    source_fingerprint,
                    Utc::now().to_rfc3339(),
                ],
            )
            .context("failed to save conversation state")?;

        Ok(())
    }

    pub fn forget_conversation(&self, provider: ProviderKind, resume_hash: &str) -> Result<bool> {
        let conversation = self
            .connection
            .query_row(
                "SELECT conversation_id FROM conversations WHERE provider = ?1 AND resume_hash = ?2",
                params![provider.as_slug(), resume_hash],
                |row| row.get::<_, String>(0),
            );

        let conversation_id = match conversation {
            Ok(value) => value,
            Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(false),
            Err(error) => return Err(error).context("failed to resolve conversation for forget"),
        };

        self.connection
            .execute(
                "INSERT INTO conversation_states
                    (provider, conversation_id, source_path, source_fingerprint, imported_at, forgotten_at, ban_reason)
                 VALUES (
                    ?1,
                    ?2,
                    COALESCE((SELECT source_path FROM conversation_states WHERE provider = ?1 AND conversation_id = ?2), ''),
                    COALESCE((SELECT source_fingerprint FROM conversation_states WHERE provider = ?1 AND conversation_id = ?2), ''),
                    COALESCE((SELECT imported_at FROM conversation_states WHERE provider = ?1 AND conversation_id = ?2), ?3),
                    ?4,
                    ?5
                 )
                 ON CONFLICT(provider, conversation_id) DO UPDATE SET
                    forgotten_at = excluded.forgotten_at,
                    ban_reason = excluded.ban_reason",
                params![
                    provider.as_slug(),
                    &conversation_id,
                    Utc::now().to_rfc3339(),
                    Utc::now().to_rfc3339(),
                    "user-forget",
                ],
            )
            .context("failed to mark conversation as forgotten")?;

        Ok(true)
    }

    pub fn is_conversation_forgotten(
        &self,
        provider: ProviderKind,
        conversation_id: &str,
    ) -> Result<bool> {
        let forgotten_at: Option<String> = self
            .connection
            .query_row(
                "SELECT forgotten_at FROM conversation_states WHERE provider = ?1 AND conversation_id = ?2",
                params![provider.as_slug(), conversation_id],
                |row| row.get(0),
            )
            .or_else(|error| match error {
                rusqlite::Error::QueryReturnedNoRows => Ok(None),
                other => Err(other),
            })
            .context("failed to read conversation state")?;

        Ok(forgotten_at.is_some())
    }

    pub fn count_conversations(&self) -> Result<i64> {
        self.connection
            .query_row("SELECT COUNT(*) FROM conversations", [], |row| row.get(0))
            .context("failed to count conversations")
    }

    pub fn count_active_conversations(&self) -> Result<i64> {
        self.connection
            .query_row(
                "SELECT COUNT(*)
                 FROM conversations c
                 LEFT JOIN conversation_states cs
                    ON cs.provider = c.provider AND cs.conversation_id = c.conversation_id
                 WHERE cs.forgotten_at IS NULL",
                [],
                |row| row.get(0),
            )
            .context("failed to count active conversations")
    }

    pub fn count_messages(&self) -> Result<i64> {
        self.connection
            .query_row("SELECT COUNT(*) FROM messages", [], |row| row.get(0))
            .context("failed to count messages")
    }

    pub fn count_message_embeddings(&self) -> Result<i64> {
        self.connection
            .query_row("SELECT COUNT(*) FROM message_embeddings", [], |row| row.get(0))
            .context("failed to count message embeddings")
    }

    pub fn schema_version(&self) -> Result<i64> {
        let raw: String = self
            .connection
            .query_row(
                "SELECT value FROM schema_meta WHERE key = 'schema_version'",
                [],
                |row| row.get(0),
            )
            .context("failed to read schema version")?;
        raw.parse::<i64>().context("schema version is not a valid integer")
    }

    pub fn list_messages(&self) -> Result<Vec<MessageRow>> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT m.provider, m.conversation_id, m.message_id, m.role, m.content
                 FROM messages m
                 LEFT JOIN conversation_states cs
                    ON cs.provider = m.provider AND cs.conversation_id = m.conversation_id
                 WHERE cs.forgotten_at IS NULL
                 ORDER BY m.provider ASC, m.conversation_id ASC, m.message_id ASC",
            )
            .context("failed to prepare message listing query")?;

        let rows = statement
            .query_map([], |row| {
                let provider_slug: String = row.get(0)?;
                let provider = ProviderKind::from_slug(&provider_slug).map_err(|error| {
                    rusqlite::Error::FromSqlConversionFailure(0, Type::Text, Box::new(error))
                })?;

                let role_slug: String = row.get(3)?;
                let role = parse_message_role(&role_slug).map_err(|error| {
                    rusqlite::Error::FromSqlConversionFailure(3, Type::Text, Box::new(error))
                })?;

                Ok(MessageRow {
                    provider,
                    conversation_id: row.get(1)?,
                    message_id: row.get(2)?,
                    role,
                    content: row.get(4)?,
                })
            })
            .context("failed to query messages")?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .context("failed to read messages")
    }

    pub fn list_recent_messages(&self, limit: usize) -> Result<Vec<MessageRow>> {
        let limit = i64::try_from(limit).context("recent message limit is too large")?;
        let mut statement = self
            .connection
            .prepare(
                "SELECT m.provider, m.conversation_id, m.message_id, m.role, m.content
                 FROM messages m
                 LEFT JOIN conversation_states cs
                    ON cs.provider = m.provider AND cs.conversation_id = m.conversation_id
                 WHERE cs.forgotten_at IS NULL
                 ORDER BY m.id DESC
                 LIMIT ?1",
            )
            .context("failed to prepare recent message listing query")?;

        let rows = statement
            .query_map(params![limit], |row| {
                let provider_slug: String = row.get(0)?;
                let provider = ProviderKind::from_slug(&provider_slug).map_err(|error| {
                    rusqlite::Error::FromSqlConversionFailure(0, Type::Text, Box::new(error))
                })?;

                let role_slug: String = row.get(3)?;
                let role = parse_message_role(&role_slug).map_err(|error| {
                    rusqlite::Error::FromSqlConversionFailure(3, Type::Text, Box::new(error))
                })?;

                Ok(MessageRow {
                    provider,
                    conversation_id: row.get(1)?,
                    message_id: row.get(2)?,
                    role,
                    content: row.get(4)?,
                })
            })
            .context("failed to query recent messages")?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .context("failed to read recent messages")
    }

    pub fn save_message(
        &self,
        provider: ProviderKind,
        conversation_id: &str,
        message_id: &str,
        role: MessageRole,
        content: &str,
    ) -> Result<()> {
        let resume_hash = cli_memory_core::derive_resume_hash(provider, conversation_id);
        self.connection
            .execute(
                "INSERT INTO conversations (provider, conversation_id, resume_hash, imported_at)
                 VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(provider, conversation_id) DO UPDATE SET
                    resume_hash = excluded.resume_hash",
                params![
                    provider.as_slug(),
                    conversation_id,
                    resume_hash,
                    Utc::now().to_rfc3339(),
                ],
            )
            .context("failed to upsert conversation while saving message")?;

        self.connection
            .execute(
                "INSERT INTO messages (provider, conversation_id, message_id, role, content)
                 VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(provider, conversation_id, message_id) DO UPDATE SET
                    role = excluded.role,
                    content = excluded.content",
                params![
                    provider.as_slug(),
                    conversation_id,
                    message_id,
                    message_role_slug(role),
                    content,
                ],
            )
            .context("failed to save message row")?;

        Ok(())
    }

    pub fn save_message_embedding(
        &self,
        provider: ProviderKind,
        conversation_id: &str,
        message_id: &str,
        embedding: &[f32],
    ) -> Result<()> {
        let embedding_json =
            serde_json::to_string(embedding).context("failed to serialize embedding vector")?;
        self.connection
            .execute(
                "INSERT INTO message_embeddings (provider, conversation_id, message_id, embedding_json)
                 VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(provider, conversation_id, message_id) DO UPDATE SET
                    embedding_json = excluded.embedding_json",
                params![
                    provider.as_slug(),
                    conversation_id,
                    message_id,
                    embedding_json,
                ],
            )
            .context("failed to save message embedding")?;

        Ok(())
    }

    pub fn list_embedded_messages(&self) -> Result<Vec<EmbeddedMessageRow>> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT m.provider, m.conversation_id, m.message_id, m.role, m.content, me.embedding_json
                 FROM messages m
                 JOIN message_embeddings me
                    ON me.provider = m.provider
                   AND me.conversation_id = m.conversation_id
                   AND me.message_id = m.message_id
                 LEFT JOIN conversation_states cs
                    ON cs.provider = m.provider AND cs.conversation_id = m.conversation_id
                 WHERE cs.forgotten_at IS NULL
                 ORDER BY m.provider ASC, m.conversation_id ASC, m.message_id ASC",
            )
            .context("failed to prepare embedded message listing query")?;

        let rows = statement
            .query_map([], |row| {
                let provider_slug: String = row.get(0)?;
                let provider = ProviderKind::from_slug(&provider_slug).map_err(|error| {
                    rusqlite::Error::FromSqlConversionFailure(0, Type::Text, Box::new(error))
                })?;

                let role_slug: String = row.get(3)?;
                let role = parse_message_role(&role_slug).map_err(|error| {
                    rusqlite::Error::FromSqlConversionFailure(3, Type::Text, Box::new(error))
                })?;
                let embedding_json: String = row.get(5)?;
                let embedding = parse_embedding(&embedding_json).map_err(|error| {
                    rusqlite::Error::FromSqlConversionFailure(5, Type::Text, Box::new(error))
                })?;

                Ok(EmbeddedMessageRow {
                    provider,
                    conversation_id: row.get(1)?,
                    message_id: row.get(2)?,
                    role,
                    content: row.get(4)?,
                    embedding,
                })
            })
            .context("failed to query embedded messages")?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .context("failed to read embedded messages")
    }

    pub fn resume_bundle(&self, resume_hash: &str) -> Result<Option<String>> {
        let conversation = self
            .connection
            .query_row(
                "SELECT c.provider, c.conversation_id
                 FROM conversations c
                 LEFT JOIN conversation_states cs
                    ON cs.provider = c.provider AND cs.conversation_id = c.conversation_id
                 WHERE c.resume_hash = ?1 AND cs.forgotten_at IS NULL",
                params![resume_hash],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            );

        let (provider, conversation_id) = match conversation {
            Ok(value) => value,
            Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
            Err(error) => return Err(error).context("failed to load conversation for resume hash"),
        };

        let mut statement = self
            .connection
            .prepare(
                "SELECT role, content
                 FROM messages
                 WHERE provider = ?1 AND conversation_id = ?2
                 ORDER BY message_id ASC",
            )
            .context("failed to prepare resume query")?;

        let rows = statement
            .query_map(params![&provider, &conversation_id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .context("failed to query resumed messages")?;

        let mut lines = Vec::new();
        for row in rows {
            let (role, content) = row?;
            lines.push(format!("{role}: {content}"));
        }

        if lines.is_empty() {
            return Ok(None);
        }

        Ok(Some(lines.join("\n")))
    }

    pub fn search_conversations(&self, query: &str, limit: usize) -> Result<Vec<String>> {
        let like = format!("%{query}%");
        let limit = i64::try_from(limit).context("search limit is too large")?;
        let mut statement = self
            .connection
            .prepare(
                "SELECT DISTINCT m.provider, m.conversation_id, m.content
                 FROM messages m
                 LEFT JOIN conversation_states cs
                    ON cs.provider = m.provider AND cs.conversation_id = m.conversation_id
                 WHERE m.content LIKE ?1 AND cs.forgotten_at IS NULL
                 ORDER BY m.provider ASC, m.conversation_id ASC
                 LIMIT ?2",
            )
            .context("failed to prepare conversation search query")?;

        let rows = statement
            .query_map(params![like, limit], |row| {
                Ok(format!(
                    "[{}:{}] {}",
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .context("failed to query conversation search results")?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .context("failed to read conversation search results")
    }

    pub fn conversation_states(&self) -> Result<Vec<ConversationStateRow>> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT provider, conversation_id, source_path, source_fingerprint, imported_at, forgotten_at, ban_reason
                 FROM conversation_states
                 ORDER BY provider ASC, conversation_id ASC",
            )
            .context("failed to prepare conversation state query")?;

        let rows = statement
            .query_map([], |row| {
                let provider_slug: String = row.get(0)?;
                let provider = ProviderKind::from_slug(&provider_slug).map_err(|error| {
                    rusqlite::Error::FromSqlConversionFailure(0, Type::Text, Box::new(error))
                })?;

                let imported_at = parse_updated_at(row.get::<_, String>(4)?)?;
                let forgotten_at = parse_optional_updated_at(row.get::<_, Option<String>>(5)?)?;

                Ok(ConversationStateRow {
                    provider,
                    conversation_id: row.get(1)?,
                    source_path: row.get(2)?,
                    source_fingerprint: row.get(3)?,
                    imported_at,
                    forgotten_at,
                    ban_reason: row.get(6)?,
                })
            })
            .context("failed to query conversation states")?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .context("failed to read conversation state rows")
    }

    pub fn delete_history(&self, provider: ProviderKind, resume_hash: &str) -> Result<bool> {
        let conversation = self
            .connection
            .query_row(
                "SELECT conversation_id FROM conversations WHERE provider = ?1 AND resume_hash = ?2",
                params![provider.as_slug(), resume_hash],
                |row| row.get::<_, String>(0),
            );

        let conversation_id = match conversation {
            Ok(value) => value,
            Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(false),
            Err(error) => return Err(error).context("failed to resolve conversation for delete"),
        };

        self.connection
            .execute(
                "DELETE FROM message_embeddings WHERE provider = ?1 AND conversation_id = ?2",
                params![provider.as_slug(), &conversation_id],
            )
            .context("failed to delete message embeddings")?;
        self.connection
            .execute(
                "DELETE FROM messages WHERE provider = ?1 AND conversation_id = ?2",
                params![provider.as_slug(), &conversation_id],
            )
            .context("failed to delete messages")?;
        self.connection
            .execute(
                "DELETE FROM conversation_states WHERE provider = ?1 AND conversation_id = ?2",
                params![provider.as_slug(), &conversation_id],
            )
            .context("failed to delete conversation state")?;
        self.connection
            .execute(
                "DELETE FROM conversations WHERE provider = ?1 AND conversation_id = ?2",
                params![provider.as_slug(), &conversation_id],
            )
            .context("failed to delete conversation")?;

        Ok(true)
    }

    pub fn clear_session(&self, provider: ProviderKind, resume_hash: &str) -> Result<bool> {
        self.delete_history(provider, resume_hash)
    }
}

fn parse_updated_at(value: String) -> rusqlite::Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(&value)
        .map(|timestamp| timestamp.with_timezone(&Utc))
        .map_err(|error| rusqlite::Error::FromSqlConversionFailure(4, Type::Text, Box::new(error)))
}

fn parse_optional_updated_at(value: Option<String>) -> rusqlite::Result<Option<DateTime<Utc>>> {
    value
        .map(parse_updated_at)
        .transpose()
}

fn message_role_slug(role: MessageRole) -> &'static str {
    match role {
        MessageRole::System => "system",
        MessageRole::User => "user",
        MessageRole::Assistant => "assistant",
        MessageRole::Tool => "tool",
    }
}

fn parse_message_role(role: &str) -> std::result::Result<MessageRole, io::Error> {
    match role {
        "system" => Ok(MessageRole::System),
        "user" => Ok(MessageRole::User),
        "assistant" => Ok(MessageRole::Assistant),
        "tool" => Ok(MessageRole::Tool),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("unknown message role: {role}"),
        )),
    }
}

fn parse_embedding(value: &str) -> std::result::Result<Vec<f32>, io::Error> {
    let parsed: Value = serde_json::from_str(value).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid embedding json: {error}"),
        )
    })?;
    let values = parsed.as_array().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "embedding json must be an array of floats",
        )
    })?;

    values
        .iter()
        .map(|item| {
            item.as_f64().map(|value| value as f32).ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "embedding json contains a non-numeric value",
                )
            })
        })
        .collect()
}
