use cli_memory_core::{ProviderKind, models::MessageRole};
use cli_memory_integrations::import_opencode;

#[test]
fn opencode_importer_extracts_messages() {
    let transcript = import_opencode(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/opencode/minimal-session.json"
    ))
    .expect("OpenCode fixture should import");

    assert_eq!(transcript.provider, ProviderKind::OpenCode);
    assert_eq!(transcript.conversation_id, "opencode-conv-1");
    assert_eq!(transcript.messages.len(), 2);
    assert_eq!(transcript.messages[0].role, MessageRole::User);
    assert_eq!(transcript.messages[1].role, MessageRole::Assistant);
}
