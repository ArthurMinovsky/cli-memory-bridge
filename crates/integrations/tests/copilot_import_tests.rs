use cli_memory_core::{ProviderKind, models::MessageRole};
use cli_memory_integrations::import_copilot;

#[test]
fn copilot_importer_extracts_events_jsonl_messages() {
    let transcript = import_copilot(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/copilot/minimal-session.jsonl"
    ))
    .expect("Copilot fixture should import");

    assert_eq!(transcript.provider, ProviderKind::Copilot);
    assert_eq!(transcript.conversation_id, "copilot-conv-1");
    assert_eq!(transcript.messages.len(), 2);
    assert_eq!(transcript.messages[0].role, MessageRole::User);
    assert_eq!(transcript.messages[1].role, MessageRole::Assistant);
}
