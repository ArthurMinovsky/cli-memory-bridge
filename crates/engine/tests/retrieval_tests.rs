use cli_memory_core::{
    ProviderKind,
    models::{ConversationLocator, ConversationTranscript, MessageRole, TranscriptMessage},
};
use cli_memory_engine::Storage;

#[test]
fn hashing_embedder_and_vector_index_can_roundtrip() {
    let embedder = cli_memory_engine::Embedder::hashing(8);
    let docs = embedder
        .embed_documents(&["database migration".into(), "vector search".into()])
        .unwrap();
    let mut index = cli_memory_engine::VectorStore::new(embedder.dimension()).unwrap();
    index.add(1, &docs[0]).unwrap();
    index.add(2, &docs[1]).unwrap();

    let query = embedder.embed_query(&["search vector".into()]).unwrap();
    let hits = index.search(&query[0], 2).unwrap();
    assert_eq!(hits[0].id, 2);
}

#[test]
fn vector_store_normalizes_scores_for_non_unit_inputs() {
    let mut index = cli_memory_engine::VectorStore::new(8).unwrap();
    index
        .add(1, &[10.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])
        .unwrap();
    index
        .add(2, &[0.0, 3.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])
        .unwrap();

    let hits = index
        .search(&[5.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], 2)
        .unwrap();
    assert_eq!(hits[0].id, 1);
    assert!(hits[0].score > hits[1].score);
    assert!(hits[0].score > 0.5);
    assert!(hits[1].score < 0.5);
}

#[test]
fn vector_store_replaces_existing_ids() {
    let mut index = cli_memory_engine::VectorStore::new(8).unwrap();
    index.add(7, &[1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]).unwrap();
    index.add(7, &[0.0, 4.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]).unwrap();

    let hits = index
        .search(&[0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], 10)
        .unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].id, 7);
    assert!(hits[0].score > 0.5);
}

#[test]
fn retrieval_service_returns_context_bundle() {
    let mut service = cli_memory_engine::test_service().unwrap();
    service
        .ingest_text("codex", "abc123", "We switched to turbovec for semantic search")
        .unwrap();
    let bundle = service.context_bundle("semantic search", 400).unwrap();
    assert!(bundle.contains("turbovec"));
}

#[test]
fn retrieval_service_respects_character_budget() {
    let mut service = cli_memory_engine::test_service().unwrap();
    service
        .ingest_text("codex", "abc123", "This bundle should be too large for the budget")
        .unwrap();

    let bundle = service.context_bundle("bundle", 10).unwrap();
    assert!(bundle.is_empty());
}

#[test]
fn zero_dimension_embedder_is_rejected() {
    let embedder = cli_memory_engine::Embedder::hashing(0);
    let error = embedder
        .embed_documents(&["text".to_owned()])
        .expect_err("zero-dimension embedder should fail");
    assert!(error.to_string().contains("greater than zero"));
}

#[test]
fn storage_backed_retrieval_service_searches_imported_messages() {
    let tempdir = tempfile::tempdir().expect("temporary directory should be created");
    let database_path = tempdir.path().join("engine.sqlite3");
    let storage = Storage::open(&database_path).expect("storage should open");
    storage.initialize().expect("schema should initialize");

    storage
        .save_transcript(&ConversationTranscript {
            locator: ConversationLocator {
                provider: ProviderKind::Codex,
                conversation_id: "conv-1".to_owned(),
            },
            messages: vec![
                TranscriptMessage {
                    message_id: "m1".to_owned(),
                    role: MessageRole::User,
                    content: "How do I install this?".to_owned(),
                },
                TranscriptMessage {
                    message_id: "m2".to_owned(),
                    role: MessageRole::Assistant,
                    content: "Use uv pip install -e .".to_owned(),
                },
            ],
        })
        .expect("transcript should save");

    let service = cli_memory_engine::RetrievalService::from_storage_with_embedder(
        &storage,
        cli_memory_engine::Embedder::hashing(128),
    )
    .expect("service should load from storage");
    let results = service.search_lines("install", 10).expect("search should succeed");

    assert!(!results.is_empty());
    assert!(results[0].contains("[codex:conv-1]"));
    assert!(results.iter().any(|line| line.contains("install")));
}
