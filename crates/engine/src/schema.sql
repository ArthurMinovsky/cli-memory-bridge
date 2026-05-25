CREATE TABLE IF NOT EXISTS schema_meta (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

INSERT INTO schema_meta (key, value)
VALUES ('schema_version', '1')
ON CONFLICT(key) DO NOTHING;

CREATE TABLE IF NOT EXISTS checkpoints (
    id INTEGER PRIMARY KEY,
    provider TEXT NOT NULL,
    source_path TEXT NOT NULL,
    fingerprint TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS checkpoints_provider_source_path_idx
    ON checkpoints (provider, source_path);

CREATE TABLE IF NOT EXISTS conversations (
    id INTEGER PRIMARY KEY,
    provider TEXT NOT NULL,
    conversation_id TEXT NOT NULL,
    resume_hash TEXT NOT NULL,
    imported_at TEXT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS conversations_provider_conversation_idx
    ON conversations (provider, conversation_id);

CREATE TABLE IF NOT EXISTS messages (
    id INTEGER PRIMARY KEY,
    provider TEXT NOT NULL,
    conversation_id TEXT NOT NULL,
    message_id TEXT NOT NULL,
    role TEXT NOT NULL,
    content TEXT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS messages_provider_conversation_message_idx
    ON messages (provider, conversation_id, message_id);

CREATE TABLE IF NOT EXISTS message_embeddings (
    id INTEGER PRIMARY KEY,
    provider TEXT NOT NULL,
    conversation_id TEXT NOT NULL,
    message_id TEXT NOT NULL,
    embedding_json TEXT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS message_embeddings_provider_conversation_message_idx
    ON message_embeddings (provider, conversation_id, message_id);

CREATE TABLE IF NOT EXISTS conversation_states (
    id INTEGER PRIMARY KEY,
    provider TEXT NOT NULL,
    conversation_id TEXT NOT NULL,
    source_path TEXT NOT NULL,
    source_fingerprint TEXT NOT NULL,
    imported_at TEXT NOT NULL,
    forgotten_at TEXT,
    ban_reason TEXT
);

CREATE UNIQUE INDEX IF NOT EXISTS conversation_states_provider_conversation_idx
    ON conversation_states (provider, conversation_id);
