use cli_memory_core::models::MessageRole;
use cli_memory_integrations::import_codex;
use std::fs;

#[test]
fn codex_importer_extracts_messages() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/codex/minimal-session.jsonl"
    );

    let transcript = import_codex(path).expect("Codex fixture should import");

    assert_eq!(transcript.provider.as_slug(), "codex");
    assert_eq!(transcript.conversation_id, "019e3c95-ac82-7402-bb65-f9bf46673f1f");
    assert_eq!(transcript.messages.len(), 2);
    assert_eq!(transcript.messages[0].role, MessageRole::User);
    assert_eq!(transcript.messages[0].content, "How do I install this?");
    assert_eq!(transcript.messages[1].role, MessageRole::Assistant);
    assert_eq!(transcript.messages[1].content, "Use uv pip install -e .");
}

#[test]
fn codex_importer_falls_back_to_file_stem_without_session_meta() {
    let dir = tempfile::tempdir().expect("temporary directory should be created");
    let path = dir.path().join("fallback-session.jsonl");
    fs::write(
        &path,
        "{\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"role\":\"assistant\",\"content\":[{\"type\":\"output_text\",\"text\":\"Recovered.\"}]}}\n",
    )
    .expect("fixture should be written");

    let transcript = import_codex(&path).expect("Codex temp fixture should import");

    assert_eq!(transcript.conversation_id, "fallback-session");
    assert_eq!(transcript.messages.len(), 1);
    assert_eq!(transcript.messages[0].content, "Recovered.");
}
