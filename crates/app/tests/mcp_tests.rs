use cli_memory_core::{
    ProviderKind,
    models::{ConversationLocator, ConversationTranscript, MessageRole, TranscriptMessage},
};
use cli_memory_engine::Storage;

#[test]
fn health_check_returns_ok() {
    let status = cli_memory_app::doctor::DoctorStatus {
        db_ready: true,
        embedder_ready: true,
        vector_ready: true,
        detected_providers: vec!["codex".to_owned(), "claude".to_owned()],
        data_dir: "/tmp/data".to_owned(),
        db_path: "/tmp/data/db.sqlite3".to_owned(),
        active_conversations: 1,
        total_conversations: 2,
        total_messages: 4,
        total_embeddings: 4,
        total_checkpoints: 3,
        model_dir: "/tmp/data/models/potion-multilingual-128M".to_owned(),
    };
    let value = cli_memory_app::mcp::health_check(&status);
    assert_eq!(value["status"], "ok");
    assert_eq!(value["db_ready"], true);
    assert_eq!(value["embedder_ready"], true);
    assert_eq!(value["vector_ready"], true);
    assert_eq!(value["detected_providers"][0], "codex");
}

#[test]
fn context_bundle_reads_from_storage() {
    let tempdir = tempfile::tempdir().expect("temporary directory should be created");
    let database_path = tempdir.path().join("engine.sqlite3");
    let storage = Storage::open(&database_path).expect("storage should open");
    storage.initialize().expect("schema should initialize");

    storage
        .save_transcript(&ConversationTranscript {
            locator: ConversationLocator {
                provider: ProviderKind::Claude,
                conversation_id: "conv-2".to_owned(),
            },
            messages: vec![
                TranscriptMessage {
                    message_id: "m1".to_owned(),
                    role: MessageRole::User,
                    content: "Need semantic retrieval help".to_owned(),
                },
                TranscriptMessage {
                    message_id: "m2".to_owned(),
                    role: MessageRole::Assistant,
                    content: "We use turbovec for semantic retrieval.".to_owned(),
                },
            ],
        })
        .expect("transcript should save");

    let value = cli_memory_app::mcp::context_bundle_with_hashing_service(
        &database_path,
        "semantic retrieval",
        400,
    )
    .expect("context bundle should load");

    assert_eq!(value["status"], "ok");
    assert_eq!(value["query"], "semantic retrieval");
    assert_eq!(value["char_budget"], 400);
    let bundle = value["bundle"].as_str().expect("bundle should be a string");
    assert!(bundle.contains("[claude:conv-2]"));
    assert!(bundle.contains("turbovec"));
}

#[test]
fn recent_history_and_stats_read_from_storage() {
    let tempdir = tempfile::tempdir().expect("temporary directory should be created");
    let database_path = tempdir.path().join("engine.sqlite3");
    let storage = Storage::open(&database_path).expect("storage should open");
    storage.initialize().expect("schema should initialize");
    storage
        .save_transcript(&ConversationTranscript {
            locator: ConversationLocator {
                provider: ProviderKind::Codex,
                conversation_id: "conv-3".to_owned(),
            },
            messages: vec![TranscriptMessage {
                message_id: "m1".to_owned(),
                role: MessageRole::User,
                content: "Need recent history".to_owned(),
            }],
        })
        .expect("transcript should save");
    storage
        .save_message_embedding(
            ProviderKind::Codex,
            "conv-3",
            "m1",
            &[0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8],
        )
        .expect("embedding should save");

    let recent = cli_memory_app::mcp::get_recent_history(&database_path, 5)
        .expect("recent history should load");
    assert_eq!(recent["status"], "ok");
    assert_eq!(recent["messages"][0]["provider"], "codex");

    let stats = cli_memory_app::mcp::memory_stats(&database_path).expect("stats should load");
    assert_eq!(stats["status"], "ok");
    assert_eq!(stats["total_conversations"], 1);
    assert_eq!(stats["total_messages"], 1);
    assert_eq!(stats["total_embeddings"], 1);
}

#[test]
fn save_turn_and_resume_round_trip() {
    let tempdir = tempfile::tempdir().expect("temporary directory should be created");
    let database_path = tempdir.path().join("engine.sqlite3");
    let storage = Storage::open(&database_path).expect("storage should open");
    storage.initialize().expect("schema should initialize");

    cli_memory_app::mcp::save_conversation_turn(
        &database_path,
        ProviderKind::Claude,
        "conv-4",
        "hello",
        "world",
    )
    .expect("turn should save");

    let hash_id = cli_memory_core::derive_resume_hash(ProviderKind::Claude, "conv-4");
    let resumed = cli_memory_app::mcp::resume_conversation(&database_path, &hash_id)
        .expect("resume should load");
    assert_eq!(resumed["status"], "ok");
    let bundle = resumed["bundle"].as_str().expect("bundle should be a string");
    assert!(bundle.contains("user: hello"));
    assert!(bundle.contains("assistant: world"));
}
