use cli_memory_core::{ProviderKind, models::MessageRole};
use cli_memory_integrations::import_antigravity_cli;

#[test]
fn antigravity_importer_extracts_markdown_artifacts() {
    let transcript = import_antigravity_cli(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/antigravity-cli/session-1"
    ))
    .expect("Antigravity CLI fixture should import");

    assert_eq!(transcript.provider, ProviderKind::AntigravityCli);
    assert_eq!(transcript.conversation_id, "session-1");
    assert_eq!(transcript.messages.len(), 3);
    assert_eq!(transcript.messages[0].role, MessageRole::User);
    assert_eq!(transcript.messages[1].role, MessageRole::Assistant);
    assert_eq!(transcript.messages[2].role, MessageRole::Assistant);
    assert!(transcript.messages[1].content.contains("Summary (ARTIFACT_TYPE_OTHER):"));
}
