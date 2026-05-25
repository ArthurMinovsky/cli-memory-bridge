use cli_memory_core::{ProviderKind, models::MessageRole};
use cli_memory_integrations::import_opencode;
use rusqlite::Connection;

#[test]
fn opencode_importer_falls_back_to_db_when_session_diff_is_empty() {
    let tempdir = tempfile::tempdir().expect("temporary directory should be created");
    let root = tempdir.path();
    std::fs::create_dir_all(root.join("storage/session_diff"))
        .expect("session diff directory should be created");
    std::fs::write(
        root.join("storage/session_diff/ses_test123.json"),
        "[]",
    )
    .expect("empty session diff should be written");

    let connection =
        Connection::open(root.join("opencode.db")).expect("temporary OpenCode db should open");
    connection
        .execute_batch(
            "
            CREATE TABLE session (
              id TEXT PRIMARY KEY,
              project_id TEXT NOT NULL,
              parent_id TEXT,
              slug TEXT NOT NULL,
              directory TEXT NOT NULL,
              title TEXT NOT NULL,
              version TEXT NOT NULL,
              share_url TEXT,
              summary_additions INTEGER,
              summary_deletions INTEGER,
              summary_files INTEGER,
              summary_diffs TEXT,
              revert TEXT,
              permission TEXT,
              time_created INTEGER NOT NULL,
              time_updated INTEGER NOT NULL,
              time_compacting INTEGER,
              time_archived INTEGER,
              workspace_id TEXT
            );
            CREATE TABLE message (
              id TEXT PRIMARY KEY,
              session_id TEXT NOT NULL,
              time_created INTEGER NOT NULL,
              time_updated INTEGER NOT NULL,
              data TEXT NOT NULL
            );
            CREATE TABLE part (
              id TEXT PRIMARY KEY,
              message_id TEXT NOT NULL,
              session_id TEXT NOT NULL,
              time_created INTEGER NOT NULL,
              time_updated INTEGER NOT NULL,
              data TEXT NOT NULL
            );
            ",
        )
        .expect("temporary OpenCode schema should initialize");

    connection
        .execute(
            "INSERT INTO session (id, project_id, slug, directory, title, version, time_created, time_updated)
             VALUES (?1, 'proj-1', 'slug', '/tmp', 'Test', '1', 1, 1)",
            ["ses_test123"],
        )
        .expect("session row should insert");
    connection
        .execute(
            "INSERT INTO message (id, session_id, time_created, time_updated, data)
             VALUES (?1, ?2, 1, 1, ?3)",
            (
                "msg-user",
                "ses_test123",
                r#"{"role":"user"}"#,
            ),
        )
        .expect("user message should insert");
    connection
        .execute(
            "INSERT INTO part (id, message_id, session_id, time_created, time_updated, data)
             VALUES (?1, ?2, ?3, 1, 1, ?4)",
            (
                "part-user",
                "msg-user",
                "ses_test123",
                r#"{"type":"text","text":"How do I run the app?"}"#,
            ),
        )
        .expect("user part should insert");
    connection
        .execute(
            "INSERT INTO message (id, session_id, time_created, time_updated, data)
             VALUES (?1, ?2, 2, 2, ?3)",
            (
                "msg-assistant",
                "ses_test123",
                r#"{"role":"assistant"}"#,
            ),
        )
        .expect("assistant message should insert");
    connection
        .execute(
            "INSERT INTO part (id, message_id, session_id, time_created, time_updated, data)
             VALUES (?1, ?2, ?3, 2, 2, ?4)",
            (
                "part-assistant",
                "msg-assistant",
                "ses_test123",
                r#"{"type":"text","text":"Use cargo run --bin cli-memory."}"#,
            ),
        )
        .expect("assistant part should insert");

    let transcript = import_opencode(root.join("storage/session_diff/ses_test123.json"))
        .expect("OpenCode DB fallback should import");

    assert_eq!(transcript.provider, ProviderKind::OpenCode);
    assert_eq!(transcript.conversation_id, "ses_test123");
    assert_eq!(transcript.messages.len(), 2);
    assert_eq!(transcript.messages[0].role, MessageRole::User);
    assert_eq!(transcript.messages[1].role, MessageRole::Assistant);
}
