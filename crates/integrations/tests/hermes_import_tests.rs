use cli_memory_core::{ProviderKind, models::MessageRole};
use cli_memory_integrations::import_hermes;

#[test]
fn hermes_importer_extracts_jsonl_messages() {
    let transcript = import_hermes(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/hermes/minimal-session.jsonl"
    ))
    .expect("Hermes fixture should import");

    assert_eq!(transcript.provider, ProviderKind::Hermes);
    assert_eq!(transcript.conversation_id, "hermes-conv-1");
    assert_eq!(transcript.messages.len(), 2);
    assert_eq!(transcript.messages[0].role, MessageRole::User);
    assert_eq!(transcript.messages[1].role, MessageRole::Assistant);
}
