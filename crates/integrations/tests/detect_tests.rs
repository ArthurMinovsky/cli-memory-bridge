use cli_memory_core::ProviderKind;
use cli_memory_integrations::detect_providers;

#[test]
fn detects_known_provider_roots() {
    let home = tempfile::tempdir().expect("temporary directory should be created");
    std::fs::create_dir_all(home.path().join(".claude/projects"))
        .expect("Claude projects directory should be created");
    std::fs::create_dir_all(home.path().join(".codex/sessions"))
        .expect("Codex sessions directory should be created");
    std::fs::write(home.path().join(".codex/session_index.jsonl"), "{}\n")
        .expect("Codex session index should be created");
    std::fs::create_dir_all(home.path().join(".gemini/tmp"))
        .expect("Gemini tmp directory should be created");
    std::fs::create_dir_all(home.path().join(".copilot/session-state"))
        .expect("Copilot session-state directory should be created");
    std::fs::create_dir_all(home.path().join(".config/zed/conversations"))
        .expect("Zed conversations directory should be created");
    std::fs::create_dir_all(home.path().join(".local/share/opencode/storage/session_diff"))
        .expect("OpenCode session_diff directory should be created");
    std::fs::create_dir_all(home.path().join(".hermes/sessions"))
        .expect("Hermes sessions directory should be created");
    std::fs::create_dir_all(home.path().join(".gemini/antigravity/brain"))
        .expect("Antigravity brain directory should be created");

    let detected = detect_providers(home.path()).expect("providers should be detected");

    assert!(detected.iter().any(|item| item.provider == ProviderKind::Claude));
    assert!(detected.iter().any(|item| item.provider == ProviderKind::Codex));
    assert!(detected.iter().any(|item| item.provider == ProviderKind::Gemini));
    assert!(detected.iter().any(|item| item.provider == ProviderKind::Copilot));
    assert!(detected.iter().any(|item| item.provider == ProviderKind::Zed));
    assert!(detected.iter().any(|item| item.provider == ProviderKind::OpenCode));
    assert!(detected.iter().any(|item| item.provider == ProviderKind::Hermes));
    assert!(detected
        .iter()
        .any(|item| item.provider == ProviderKind::AntigravityCli));
}

#[test]
fn returns_empty_when_no_known_roots_exist() {
    let home = tempfile::tempdir().expect("temporary directory should be created");
    let detected = detect_providers(home.path()).expect("provider scan should succeed");
    assert!(detected.is_empty());
}
