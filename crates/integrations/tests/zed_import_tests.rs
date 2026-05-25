use cli_memory_core::{ProviderKind, models::MessageRole};
use cli_memory_integrations::import_zed;

#[test]
fn zed_importer_extracts_summary_and_messages() {
    let transcript = import_zed(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/zed/minimal-session.zed.json"
    ))
    .expect("Zed fixture should import");

    assert_eq!(transcript.provider, ProviderKind::Zed);
    assert_eq!(transcript.conversation_id, "zed-conv-1");
    assert_eq!(transcript.messages.len(), 3);
    assert_eq!(transcript.messages[0].role, MessageRole::System);
    assert_eq!(transcript.messages[1].role, MessageRole::User);
    assert_eq!(transcript.messages[2].role, MessageRole::Assistant);
}

#[test]
fn zed_importer_skips_summary_only_conversations() {
    let transcript = import_zed(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/zed/summary-only.zed.json"
    ))
    .expect("Zed summary-only fixture should parse");

    assert_eq!(transcript.provider, ProviderKind::Zed);
    assert_eq!(transcript.conversation_id, "zed-summary-only");
    assert!(transcript.messages.is_empty());
}
