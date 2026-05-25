use cli_memory_core::{ProviderKind, models::MessageRole};
use cli_memory_integrations::import_gemini;

#[test]
fn gemini_importer_extracts_user_and_assistant_messages() {
    let transcript = import_gemini(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/gemini/minimal-session.json"
    ))
    .expect("Gemini fixture should import");

    assert_eq!(transcript.provider, ProviderKind::Gemini);
    assert_eq!(transcript.conversation_id, "gemini-conv-1");
    assert_eq!(transcript.messages.len(), 2);
    assert_eq!(transcript.messages[0].role, MessageRole::User);
    assert_eq!(transcript.messages[1].role, MessageRole::Assistant);
}
