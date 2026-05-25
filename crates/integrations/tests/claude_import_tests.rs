use cli_memory_core::models::MessageRole;
use cli_memory_integrations::import_claude;
use std::fs;

#[test]
fn claude_importer_extracts_text_and_tool_messages() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/claude/minimal-session.jsonl"
    );

    let transcript = import_claude(path).expect("Claude fixture should import");

    assert_eq!(transcript.provider.as_slug(), "claude");
    assert_eq!(transcript.conversation_id, "e9639077-a365-4bb2-86f4-4f77ba578ef0");
    assert_eq!(transcript.messages.len(), 3);
    assert_eq!(transcript.messages[0].role, MessageRole::User);
    assert_eq!(transcript.messages[0].content, "Find the cause.");
    assert_eq!(transcript.messages[1].role, MessageRole::Tool);
    assert_eq!(transcript.messages[1].content, "Bash: {\"command\":\"docker ps -a\"}");
    assert_eq!(transcript.messages[2].role, MessageRole::Assistant);
    assert_eq!(transcript.messages[2].content, "All clean.");
}

#[test]
fn claude_importer_falls_back_to_file_stem_and_reads_tool_result_strings() {
    let dir = tempfile::tempdir().expect("temporary directory should be created");
    let path = dir.path().join("claude-fallback.jsonl");
    fs::write(
        &path,
        "{\"type\":\"assistant\",\"message\":{\"role\":\"assistant\",\"content\":[{\"type\":\"tool_result\",\"content\":\"Tool finished.\"}]}}\n",
    )
    .expect("fixture should be written");

    let transcript = import_claude(&path).expect("Claude temp fixture should import");

    assert_eq!(transcript.conversation_id, "claude-fallback");
    assert_eq!(transcript.messages.len(), 1);
    assert_eq!(transcript.messages[0].role, MessageRole::Tool);
    assert_eq!(transcript.messages[0].content, "Tool finished.");
}

#[test]
fn claude_importer_skips_progress_and_snapshot_rows() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/claude/mixed-events-session.jsonl"
    );

    let transcript = import_claude(path).expect("Claude mixed-event fixture should import");

    assert_eq!(transcript.conversation_id, "real-claude-session-1");
    assert_eq!(transcript.messages.len(), 2);
    assert_eq!(transcript.messages[0].role, MessageRole::User);
    assert_eq!(transcript.messages[0].content, "Find the root cause first.");
    assert_eq!(transcript.messages[1].role, MessageRole::Assistant);
    assert_eq!(
        transcript.messages[1].content,
        "I will inspect the transcript format."
    );
}
