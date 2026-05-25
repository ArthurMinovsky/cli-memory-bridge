use chrono::{TimeZone, Utc};
use cli_memory_core::ProviderKind;
use cli_memory_core::{
    derive_resume_hash,
    models::{ConversationLocator, ConversationTranscript, MessageRole, TranscriptMessage},
};
use cli_memory_engine::{Checkpoint, Storage};

#[test]
fn open_initialize_save_and_list_checkpoint_round_trips_source_path_and_fingerprint() {
    let tempdir = tempfile::tempdir().expect("temporary directory should be created");
    let database_path = tempdir.path().join("engine.sqlite3");

    let storage = Storage::open(&database_path).expect("storage should open");
    storage
        .initialize()
        .expect("schema should initialize successfully");

    let checkpoint = Checkpoint {
        provider: ProviderKind::Codex,
        source_path: "/imports/session-a.jsonl".to_owned(),
        fingerprint: "sha256:abc123".to_owned(),
        updated_at: Utc
            .with_ymd_and_hms(2026, 5, 24, 8, 0, 0)
            .single()
            .expect("timestamp should be valid"),
    };

    storage
        .save_checkpoint(&checkpoint)
        .expect("checkpoint should save successfully");

    let checkpoints = storage
        .list_checkpoints()
        .expect("saved checkpoints should be listed");

    assert_eq!(checkpoints.len(), 1);
    assert_eq!(checkpoints[0].provider, ProviderKind::Codex);
    assert_eq!(checkpoints[0].source_path, checkpoint.source_path);
    assert_eq!(checkpoints[0].fingerprint, checkpoint.fingerprint);
    assert_eq!(checkpoints[0].updated_at, checkpoint.updated_at);
}

#[test]
fn save_checkpoint_updates_existing_provider_and_source_path() {
    let tempdir = tempfile::tempdir().expect("temporary directory should be created");
    let database_path = tempdir.path().join("engine.sqlite3");

    let storage = Storage::open(&database_path).expect("storage should open");
    storage
        .initialize()
        .expect("schema should initialize successfully");

    let original = Checkpoint {
        provider: ProviderKind::Claude,
        source_path: "/imports/session-b.jsonl".to_owned(),
        fingerprint: "sha256:old".to_owned(),
        updated_at: Utc
            .with_ymd_and_hms(2026, 5, 24, 9, 0, 0)
            .single()
            .expect("timestamp should be valid"),
    };

    let updated = Checkpoint {
        provider: ProviderKind::Claude,
        source_path: original.source_path.clone(),
        fingerprint: "sha256:new".to_owned(),
        updated_at: Utc
            .with_ymd_and_hms(2026, 5, 24, 10, 30, 0)
            .single()
            .expect("timestamp should be valid"),
    };

    storage
        .save_checkpoint(&original)
        .expect("original checkpoint should save successfully");

    let original_id = storage
        .list_checkpoints()
        .expect("original checkpoint should be listed")[0]
        .id;

    storage
        .save_checkpoint(&updated)
        .expect("updated checkpoint should save successfully");

    let checkpoints = storage
        .list_checkpoints()
        .expect("saved checkpoints should be listed");

    assert_eq!(checkpoints.len(), 1);
    assert_eq!(checkpoints[0].id, original_id);
    assert_eq!(checkpoints[0].provider, ProviderKind::Claude);
    assert_eq!(checkpoints[0].source_path, updated.source_path);
    assert_eq!(checkpoints[0].fingerprint, updated.fingerprint);
    assert_eq!(checkpoints[0].updated_at, updated.updated_at);
}

#[test]
fn checkpoints_persist_after_reopen() {
    let tempdir = tempfile::tempdir().expect("temporary directory should be created");
    let database_path = tempdir.path().join("engine.sqlite3");

    {
        let storage = Storage::open(&database_path).expect("storage should open");
        storage
            .initialize()
            .expect("schema should initialize successfully");

        let checkpoint = Checkpoint {
            provider: ProviderKind::Codex,
            source_path: "/imports/session-c.jsonl".to_owned(),
            fingerprint: "sha256:persist".to_owned(),
            updated_at: Utc
                .with_ymd_and_hms(2026, 5, 24, 11, 45, 0)
                .single()
                .expect("timestamp should be valid"),
        };

        storage
            .save_checkpoint(&checkpoint)
            .expect("checkpoint should save successfully");
    }

    let reopened = Storage::open(&database_path).expect("storage should reopen");
    let checkpoints = reopened
        .list_checkpoints()
        .expect("saved checkpoints should still be listed");

    assert_eq!(checkpoints.len(), 1);
    assert_eq!(checkpoints[0].provider, ProviderKind::Codex);
    assert_eq!(checkpoints[0].source_path, "/imports/session-c.jsonl");
    assert_eq!(checkpoints[0].fingerprint, "sha256:persist");
}

#[test]
fn initialize_is_idempotent_and_sets_schema_version() {
    let tempdir = tempfile::tempdir().expect("temporary directory should be created");
    let database_path = tempdir.path().join("engine.sqlite3");

    let storage = Storage::open(&database_path).expect("storage should open");
    storage
        .initialize()
        .expect("schema should initialize successfully");
    storage
        .initialize()
        .expect("schema should initialize successfully on rerun");

    assert_eq!(storage.schema_version().expect("schema version should load"), 1);
}

#[test]
fn forgotten_conversations_are_excluded_from_active_message_and_resume_views() {
    let tempdir = tempfile::tempdir().expect("temporary directory should be created");
    let database_path = tempdir.path().join("engine.sqlite3");

    let storage = Storage::open(&database_path).expect("storage should open");
    storage
        .initialize()
        .expect("schema should initialize successfully");

    storage
        .save_transcript(&ConversationTranscript {
            locator: ConversationLocator {
                provider: ProviderKind::Codex,
                conversation_id: "conv-1".to_owned(),
            },
            messages: vec![TranscriptMessage {
                message_id: "m1".to_owned(),
                role: MessageRole::User,
                content: "forget me".to_owned(),
            }],
        })
        .expect("transcript should save");
    storage
        .save_conversation_state(
            ProviderKind::Codex,
            "conv-1",
            "/imports/session.jsonl",
            "file:123:456",
        )
        .expect("conversation state should save");

    let hash_id = derive_resume_hash(ProviderKind::Codex, "conv-1");
    assert!(storage
        .forget_conversation(ProviderKind::Codex, &hash_id)
        .expect("forget should succeed"));

    assert!(storage
        .list_messages()
        .expect("messages should list")
        .is_empty());
    assert!(storage
        .resume_bundle(&hash_id)
        .expect("resume should query")
        .is_none());
    assert!(storage
        .search_conversations("forget", 10)
        .expect("search should query")
        .is_empty());
}

#[test]
fn resume_bundle_preserves_numeric_message_order() {
    let tempdir = tempfile::tempdir().expect("temporary directory should be created");
    let database_path = tempdir.path().join("engine.sqlite3");

    let storage = Storage::open(&database_path).expect("storage should open");
    storage
        .initialize()
        .expect("schema should initialize successfully");

    let messages = (1..=10)
        .map(|index| TranscriptMessage {
            message_id: format!("msg-{index}"),
            role: if index % 2 == 0 {
                MessageRole::Assistant
            } else {
                MessageRole::User
            },
            content: format!("line {index}"),
        })
        .collect::<Vec<_>>();

    storage
        .save_transcript(&ConversationTranscript {
            locator: ConversationLocator {
                provider: ProviderKind::Codex,
                conversation_id: "conv-ordered".to_owned(),
            },
            messages,
        })
        .expect("transcript should save");

    let hash_id = derive_resume_hash(ProviderKind::Codex, "conv-ordered");
    let bundle = storage
        .resume_bundle(&hash_id)
        .expect("resume should query")
        .expect("resume bundle should exist");

    let line_2 = bundle.find("line 2").expect("line 2 should exist");
    let line_10 = bundle.find("line 10").expect("line 10 should exist");
    assert!(line_2 < line_10);
}
