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

#[test]
fn copilot_importer_extracts_real_user_and_assistant_message_events() {
    let transcript = import_copilot(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/copilot/real-events-session.jsonl"
    ))
    .expect("Copilot real-event fixture should import");

    assert_eq!(transcript.provider, ProviderKind::Copilot);
    assert_eq!(
        transcript.conversation_id,
        "21770214-6dae-47e8-b12c-26baa0be194c"
    );
    assert_eq!(transcript.messages.len(), 2);
    assert_eq!(transcript.messages[0].role, MessageRole::User);
    assert_eq!(transcript.messages[0].content, "how to update copolot?");
    assert_eq!(transcript.messages[1].role, MessageRole::Assistant);
    assert!(transcript.messages[1].content.contains("run `/update` inside the CLI"));
}
