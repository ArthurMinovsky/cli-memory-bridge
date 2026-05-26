use chrono::Utc;
use cli_memory_engine::{Checkpoint, Storage};
use cli_memory_core::{ProviderKind, derive_resume_hash};

#[test]
fn binary_reports_help() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_cli-memory"))
        .arg("--help")
        .output()
        .expect("binary should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("cli-memory"));
    assert!(stdout.contains("init"));
}

#[test]
fn help_lists_resume_and_conv_search() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_cli-memory"))
        .arg("--help")
        .output()
        .expect("binary should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("resume"));
    assert!(stdout.contains("conv-search"));
    assert!(stdout.contains("forget"));
    assert!(stdout.contains("install"));
    assert!(stdout.contains("unlink"));
    assert!(stdout.contains("uninstall"));
    assert!(stdout.contains("serve"));
    assert!(stdout.contains("stats"));
}

#[test]
fn codex_resume_skill_contains_resume_instruction() {
    let text = cli_memory_integrations::render_codex_resume_skill();
    assert!(text.contains("$resume"));
    assert!(text.contains("`resume`"));
}

#[test]
fn codex_forget_skill_contains_provider_scoped_instruction() {
    let text = cli_memory_integrations::render_codex_forget_skill();
    assert!(text.contains("$forget"));
    assert!(text.contains("cli-memory forget codex <hash-id>"));
}

#[test]
fn codex_conv_search_skill_contains_search_instruction() {
    let text = cli_memory_integrations::render_codex_conv_search_skill();
    assert!(text.contains("$conv-search"));
    assert!(text.contains("`conv-search`"));
}

#[test]
fn claude_conv_search_command_contains_search_instruction() {
    let text = cli_memory_integrations::render_claude_conv_search_command();
    assert!(text.contains("/conv-search"));
    assert!(text.contains("`conv-search`"));
}

#[test]
fn zed_installer_renders_context_server() {
    let json = cli_memory_integrations::render_zed_install("/tmp/cli-memory");
    assert!(json.contains("\"context_servers\""));
    assert!(json.contains("cli-memory"));
    assert!(json.contains("/tmp/cli-memory"));
}

#[test]
fn hermes_installer_renders_yaml_config() {
    let yaml = cli_memory_integrations::render_hermes_install("/tmp/cli-memory");
    assert!(yaml.contains("mcp_servers:"));
    assert!(yaml.contains("cli-memory:"));
    assert!(yaml.contains("/tmp/cli-memory"));
}

#[test]
fn install_command_renders_gemini_bundle() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_cli-memory"))
        .args(["install", "gemini"])
        .output()
        .expect("install should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"provider\": \"gemini\""));
    assert!(stdout.contains("\"config_path\": \"~/.gemini/settings.json\""));
    assert!(stdout.contains("\"preferred_launcher\": \"npx\""));
    assert!(stdout.contains("npx"));
    assert!(stdout.contains("cli-memory"));
    assert!(stdout.contains("\"binary_snippet\""));
}

#[test]
fn install_command_renders_copilot_bundle_with_documented_config_path() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_cli-memory"))
        .args(["install", "copilot"])
        .output()
        .expect("install should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"provider\": \"copilot\""));
    assert!(stdout.contains("\"config_path\": \"~/.copilot/mcp-config.json\""));
    assert!(stdout.contains("\"preferred_launcher\": \"global-command\""));
    assert!(stdout.contains("\\\"command\\\":\\\"cli-memory\\\""));
    assert!(stdout.contains("\\\"args\\\":[\\\"serve\\\"]"));
}

#[test]
fn install_all_command_renders_all_provider_bundles() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_cli-memory"))
        .args(["install", "--all"])
        .output()
        .expect("install --all should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"mode\": \"install-all\""));
    assert!(stdout.contains("\"provider\": \"gemini\""));
    assert!(stdout.contains("\"provider\": \"antigravity-cli\""));
}

#[test]
fn install_command_renders_codex_bundle() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_cli-memory"))
        .args(["install", "codex"])
        .output()
        .expect("install should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"provider\": \"codex\""));
    assert!(stdout.contains("\"config_path\": \"~/.codex/config.toml\""));
    assert!(stdout.contains("\"preferred_launcher\": \"binary-path\""));
    assert!(stdout.contains("[mcp_servers.cli-memory]"));
    assert!(stdout.contains("startup_timeout_sec = 120"));
    assert!(stdout.contains("$resume"));
    assert!(stdout.contains("$conv-search"));
    assert!(stdout.contains("$forget"));
}

#[test]
fn unlink_command_renders_provider_cleanup_bundle() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_cli-memory"))
        .args(["unlink", "gemini"])
        .output()
        .expect("unlink should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"provider\": \"gemini\""));
    assert!(stdout.contains("\"config_path\": \"~/.gemini/settings.json\""));
    assert!(stdout.contains("\"remove_keys\""));
}

#[test]
fn unlink_all_command_renders_all_provider_cleanup_bundles() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_cli-memory"))
        .args(["unlink", "--all"])
        .output()
        .expect("unlink --all should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"mode\": \"unlink-all\""));
    assert!(stdout.contains("\"provider\": \"codex\""));
    assert!(stdout.contains("\"provider\": \"antigravity-cli\""));
}

#[test]
fn uninstall_command_renders_package_removal_guidance() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_cli-memory"))
        .arg("uninstall")
        .output()
        .expect("uninstall should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"mode\": \"uninstall\""));
    assert!(stdout.contains("cli-memory unlink --all"));
    assert!(stdout.contains("npm uninstall -g @aminovsky/cli-memory"));
}

#[test]
fn resume_prints_hash_id() {
    let home = tempfile::tempdir().expect("temporary directory should be created");
    let data_dir = tempfile::tempdir().expect("temporary data directory should be created");
    let model_dir = tempfile::tempdir().expect("temporary model directory should be created");
    let codex_session = home.path().join(".codex/sessions/session.jsonl");

    std::fs::create_dir_all(home.path().join(".codex/sessions"))
        .expect("Codex sessions directory should be created");
    std::fs::write(home.path().join(".codex/session_index.jsonl"), "{}\n")
        .expect("Codex session index should be written");
    std::fs::copy(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/codex/minimal-session.jsonl"
        ),
        &codex_session,
    )
    .expect("Codex fixture should be copied");

    let init = std::process::Command::new(env!("CARGO_BIN_EXE_cli-memory"))
        .arg("init")
        .env("CLI_MEMORY_HOME", home.path())
        .env("CLI_MEMORY_DATA_DIR", data_dir.path())
        .env("CLI_MEMORY_MODEL_PATH", model_dir.path())
        .output()
        .expect("init should run");
    assert!(init.status.success());

    let hash_id = derive_resume_hash(
        ProviderKind::Codex,
        "019e3c95-ac82-7402-bb65-f9bf46673f1f",
    );
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_cli-memory"))
        .args(["resume", &hash_id])
        .env("CLI_MEMORY_HOME", home.path())
        .env("CLI_MEMORY_DATA_DIR", data_dir.path())
        .env("CLI_MEMORY_MODEL_PATH", model_dir.path())
        .output()
        .expect("resume should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("user: How do I install this?"));
    assert!(stdout.contains("assistant: Use uv pip install -e ."));
}

#[test]
fn conv_search_prints_query() {
    let home = tempfile::tempdir().expect("temporary directory should be created");
    let data_dir = tempfile::tempdir().expect("temporary data directory should be created");
    let model_dir = tempfile::tempdir().expect("temporary model directory should be created");
    let codex_session = home.path().join(".codex/sessions/session.jsonl");

    std::fs::create_dir_all(home.path().join(".codex/sessions"))
        .expect("Codex sessions directory should be created");
    std::fs::write(home.path().join(".codex/session_index.jsonl"), "{}\n")
        .expect("Codex session index should be written");
    std::fs::copy(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/codex/minimal-session.jsonl"
        ),
        &codex_session,
    )
    .expect("Codex fixture should be copied");

    let init = std::process::Command::new(env!("CARGO_BIN_EXE_cli-memory"))
        .arg("init")
        .env("CLI_MEMORY_HOME", home.path())
        .env("CLI_MEMORY_DATA_DIR", data_dir.path())
        .env("CLI_MEMORY_MODEL_PATH", model_dir.path())
        .output()
        .expect("init should run");
    assert!(init.status.success());

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_cli-memory"))
        .args(["conv-search", "install"])
        .env("CLI_MEMORY_HOME", home.path())
        .env("CLI_MEMORY_DATA_DIR", data_dir.path())
        .env("CLI_MEMORY_MODEL_PATH", model_dir.path())
        .output()
        .expect("conv-search should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("[codex:019e3c95-ac82-7402-bb65-f9bf46673f1f]"));
    assert!(stdout.contains("How do I install this?"));
}

#[test]
fn init_imports_gemini_sessions_into_resume_and_search() {
    let home = tempfile::tempdir().expect("temporary directory should be created");
    let data_dir = tempfile::tempdir().expect("temporary data directory should be created");
    let model_dir = tempfile::tempdir().expect("temporary model directory should be created");
    let gemini_session = home
        .path()
        .join(".gemini/tmp/desktop/chats/session-2026-05-05T14-43-356dbecc.json");

    std::fs::create_dir_all(
        home.path().join(".gemini/tmp/desktop/chats"),
    )
    .expect("Gemini chats directory should be created");
    std::fs::copy(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/gemini/minimal-session.json"
        ),
        &gemini_session,
    )
    .expect("Gemini fixture should be copied");

    let init = std::process::Command::new(env!("CARGO_BIN_EXE_cli-memory"))
        .arg("init")
        .env("CLI_MEMORY_HOME", home.path())
        .env("CLI_MEMORY_DATA_DIR", data_dir.path())
        .env("CLI_MEMORY_MODEL_PATH", model_dir.path())
        .output()
        .expect("init should run");
    assert!(init.status.success());

    let hash_id = derive_resume_hash(ProviderKind::Gemini, "gemini-conv-1");
    let resume = std::process::Command::new(env!("CARGO_BIN_EXE_cli-memory"))
        .args(["resume", &hash_id])
        .env("CLI_MEMORY_HOME", home.path())
        .env("CLI_MEMORY_DATA_DIR", data_dir.path())
        .env("CLI_MEMORY_MODEL_PATH", model_dir.path())
        .output()
        .expect("resume should run");
    assert!(resume.status.success());
    let resume_stdout = String::from_utf8_lossy(&resume.stdout);
    assert!(resume_stdout.contains("user: How do I run the app?"));
    assert!(resume_stdout.contains("assistant: Use cargo run --bin cli-memory."));

    let search = std::process::Command::new(env!("CARGO_BIN_EXE_cli-memory"))
        .args(["conv-search", "run the app"])
        .env("CLI_MEMORY_HOME", home.path())
        .env("CLI_MEMORY_DATA_DIR", data_dir.path())
        .env("CLI_MEMORY_MODEL_PATH", model_dir.path())
        .output()
        .expect("conv-search should run");
    assert!(search.status.success());
    assert!(String::from_utf8_lossy(&search.stdout).contains("[gemini:gemini-conv-1]"));
}

#[test]
fn init_imports_copilot_sessions_into_resume_and_search() {
    let home = tempfile::tempdir().expect("temporary directory should be created");
    let data_dir = tempfile::tempdir().expect("temporary data directory should be created");
    let model_dir = tempfile::tempdir().expect("temporary model directory should be created");
    let copilot_session = home
        .path()
        .join(".copilot/session-state/00015301-8a7f-4967-9ffe-48778191172e/events.jsonl");

    std::fs::create_dir_all(
        home.path().join(".copilot/session-state/00015301-8a7f-4967-9ffe-48778191172e"),
    )
    .expect("Copilot session-state directory should be created");
    std::fs::copy(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/copilot/minimal-session.jsonl"
        ),
        &copilot_session,
    )
    .expect("Copilot fixture should be copied");

    let init = std::process::Command::new(env!("CARGO_BIN_EXE_cli-memory"))
        .arg("init")
        .env("CLI_MEMORY_HOME", home.path())
        .env("CLI_MEMORY_DATA_DIR", data_dir.path())
        .env("CLI_MEMORY_MODEL_PATH", model_dir.path())
        .output()
        .expect("init should run");
    assert!(init.status.success());

    let hash_id = derive_resume_hash(ProviderKind::Copilot, "copilot-conv-1");
    let resume = std::process::Command::new(env!("CARGO_BIN_EXE_cli-memory"))
        .args(["resume", &hash_id])
        .env("CLI_MEMORY_HOME", home.path())
        .env("CLI_MEMORY_DATA_DIR", data_dir.path())
        .env("CLI_MEMORY_MODEL_PATH", model_dir.path())
        .output()
        .expect("resume should run");
    assert!(resume.status.success());
    let resume_stdout = String::from_utf8_lossy(&resume.stdout);
    assert!(resume_stdout.contains("user: How do I run the app?"));
    assert!(resume_stdout.contains("assistant: Use cargo run --bin cli-memory."));

    let search = std::process::Command::new(env!("CARGO_BIN_EXE_cli-memory"))
        .args(["conv-search", "run the app"])
        .env("CLI_MEMORY_HOME", home.path())
        .env("CLI_MEMORY_DATA_DIR", data_dir.path())
        .env("CLI_MEMORY_MODEL_PATH", model_dir.path())
        .output()
        .expect("conv-search should run");
    assert!(search.status.success());
    assert!(String::from_utf8_lossy(&search.stdout).contains("[copilot:copilot-conv-1]"));
}

#[test]
fn init_imports_zed_sessions_into_resume_and_search() {
    let home = tempfile::tempdir().expect("temporary directory should be created");
    let data_dir = tempfile::tempdir().expect("temporary data directory should be created");
    let model_dir = tempfile::tempdir().expect("temporary model directory should be created");
    let zed_session = home
        .path()
        .join(".config/zed/conversations/LLM Project Workflow and Testing - 2.zed.json");

    std::fs::create_dir_all(home.path().join(".config/zed/conversations"))
        .expect("Zed conversations directory should be created");
    std::fs::copy(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/zed/minimal-session.zed.json"
        ),
        &zed_session,
    )
    .expect("Zed fixture should be copied");

    let init = std::process::Command::new(env!("CARGO_BIN_EXE_cli-memory"))
        .arg("init")
        .env("CLI_MEMORY_HOME", home.path())
        .env("CLI_MEMORY_DATA_DIR", data_dir.path())
        .env("CLI_MEMORY_MODEL_PATH", model_dir.path())
        .output()
        .expect("init should run");
    assert!(init.status.success());

    let hash_id = derive_resume_hash(ProviderKind::Zed, "zed-conv-1");
    let resume = std::process::Command::new(env!("CARGO_BIN_EXE_cli-memory"))
        .args(["resume", &hash_id])
        .env("CLI_MEMORY_HOME", home.path())
        .env("CLI_MEMORY_DATA_DIR", data_dir.path())
        .env("CLI_MEMORY_MODEL_PATH", model_dir.path())
        .output()
        .expect("resume should run");
    assert!(resume.status.success());
    let resume_stdout = String::from_utf8_lossy(&resume.stdout);
    assert!(resume_stdout.contains("Summary: Run the app"));
    assert!(resume_stdout.contains("assistant: Use cargo run --bin cli-memory."));

    let search = std::process::Command::new(env!("CARGO_BIN_EXE_cli-memory"))
        .args(["conv-search", "run the app"])
        .env("CLI_MEMORY_HOME", home.path())
        .env("CLI_MEMORY_DATA_DIR", data_dir.path())
        .env("CLI_MEMORY_MODEL_PATH", model_dir.path())
        .output()
        .expect("conv-search should run");
    assert!(search.status.success());
    assert!(String::from_utf8_lossy(&search.stdout).contains("[zed:zed-conv-1]"));
}

#[test]
fn init_skips_low_content_zed_conversations() {
    let home = tempfile::tempdir().expect("temporary directory should be created");
    let data_dir = tempfile::tempdir().expect("temporary data directory should be created");
    let model_dir = tempfile::tempdir().expect("temporary model directory should be created");
    let zed_session = home
        .path()
        .join(".config/zed/conversations/summary-only.zed.json");

    std::fs::create_dir_all(home.path().join(".config/zed/conversations"))
        .expect("Zed conversations directory should be created");
    std::fs::copy(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/zed/summary-only.zed.json"
        ),
        &zed_session,
    )
    .expect("Zed summary-only fixture should be copied");

    let init = std::process::Command::new(env!("CARGO_BIN_EXE_cli-memory"))
        .arg("init")
        .env("CLI_MEMORY_HOME", home.path())
        .env("CLI_MEMORY_DATA_DIR", data_dir.path())
        .env("CLI_MEMORY_MODEL_PATH", model_dir.path())
        .output()
        .expect("init should run");
    assert!(init.status.success());

    let storage = Storage::open(data_dir.path().join("db.sqlite3")).expect("db should open");
    assert_eq!(storage.count_conversations().expect("conversations should count"), 0);
    assert_eq!(storage.count_messages().expect("messages should count"), 0);
}

#[test]
fn init_imports_opencode_sessions_into_resume_and_search() {
    let home = tempfile::tempdir().expect("temporary directory should be created");
    let data_dir = tempfile::tempdir().expect("temporary data directory should be created");
    let model_dir = tempfile::tempdir().expect("temporary model directory should be created");
    let opencode_session = home
        .path()
        .join(".local/share/opencode/storage/session_diff/ses_test123.json");

    std::fs::create_dir_all(home.path().join(".local/share/opencode/storage/session_diff"))
        .expect("OpenCode session_diff directory should be created");
    std::fs::copy(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/opencode/minimal-session.json"
        ),
        &opencode_session,
    )
    .expect("OpenCode fixture should be copied");

    let init = std::process::Command::new(env!("CARGO_BIN_EXE_cli-memory"))
        .arg("init")
        .env("CLI_MEMORY_HOME", home.path())
        .env("CLI_MEMORY_DATA_DIR", data_dir.path())
        .env("CLI_MEMORY_MODEL_PATH", model_dir.path())
        .output()
        .expect("init should run");
    assert!(init.status.success());

    let hash_id = derive_resume_hash(ProviderKind::OpenCode, "opencode-conv-1");
    let resume = std::process::Command::new(env!("CARGO_BIN_EXE_cli-memory"))
        .args(["resume", &hash_id])
        .env("CLI_MEMORY_HOME", home.path())
        .env("CLI_MEMORY_DATA_DIR", data_dir.path())
        .env("CLI_MEMORY_MODEL_PATH", model_dir.path())
        .output()
        .expect("resume should run");
    assert!(resume.status.success());
    let resume_stdout = String::from_utf8_lossy(&resume.stdout);
    assert!(resume_stdout.contains("user: How do I run the app?"));
    assert!(resume_stdout.contains("assistant: Use cargo run --bin cli-memory."));

    let search = std::process::Command::new(env!("CARGO_BIN_EXE_cli-memory"))
        .args(["conv-search", "run the app"])
        .env("CLI_MEMORY_HOME", home.path())
        .env("CLI_MEMORY_DATA_DIR", data_dir.path())
        .env("CLI_MEMORY_MODEL_PATH", model_dir.path())
        .output()
        .expect("conv-search should run");
    assert!(search.status.success());
    assert!(String::from_utf8_lossy(&search.stdout).contains("[opencode:opencode-conv-1]"));
}

#[test]
fn init_imports_antigravity_sessions_into_resume_and_search() {
    let home = tempfile::tempdir().expect("temporary directory should be created");
    let data_dir = tempfile::tempdir().expect("temporary data directory should be created");
    let model_dir = tempfile::tempdir().expect("temporary model directory should be created");
    let antigravity_dir = home
        .path()
        .join(".gemini/antigravity/brain/session-1");

    std::fs::create_dir_all(&antigravity_dir)
        .expect("Antigravity brain session directory should be created");
    std::fs::copy(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/antigravity-cli/session-1/task.md"
        ),
        antigravity_dir.join("task.md"),
    )
    .expect("Antigravity task fixture should be copied");
    std::fs::copy(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/antigravity-cli/session-1/task.md.metadata.json"
        ),
        antigravity_dir.join("task.md.metadata.json"),
    )
    .expect("Antigravity task metadata fixture should be copied");
    std::fs::copy(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/antigravity-cli/session-1/implementation_plan.md"
        ),
        antigravity_dir.join("implementation_plan.md"),
    )
    .expect("Antigravity implementation plan fixture should be copied");

    let init = std::process::Command::new(env!("CARGO_BIN_EXE_cli-memory"))
        .arg("init")
        .env("CLI_MEMORY_HOME", home.path())
        .env("CLI_MEMORY_DATA_DIR", data_dir.path())
        .env("CLI_MEMORY_MODEL_PATH", model_dir.path())
        .output()
        .expect("init should run");
    assert!(init.status.success());

    let hash_id = derive_resume_hash(ProviderKind::AntigravityCli, "session-1");
    let resume = std::process::Command::new(env!("CARGO_BIN_EXE_cli-memory"))
        .args(["resume", &hash_id])
        .env("CLI_MEMORY_HOME", home.path())
        .env("CLI_MEMORY_DATA_DIR", data_dir.path())
        .env("CLI_MEMORY_MODEL_PATH", model_dir.path())
        .output()
        .expect("resume should run");
    assert!(resume.status.success());
    let resume_stdout = String::from_utf8_lossy(&resume.stdout);
    assert!(resume_stdout.contains("user: # Task"));
    assert!(resume_stdout.contains("assistant: Summary (ARTIFACT_TYPE_OTHER): Ask how to run the app."));

    let search = std::process::Command::new(env!("CARGO_BIN_EXE_cli-memory"))
        .args(["conv-search", "run the app"])
        .env("CLI_MEMORY_HOME", home.path())
        .env("CLI_MEMORY_DATA_DIR", data_dir.path())
        .env("CLI_MEMORY_MODEL_PATH", model_dir.path())
        .output()
        .expect("conv-search should run");
    assert!(search.status.success());
    assert!(String::from_utf8_lossy(&search.stdout).contains("[antigravity-cli:session-1]"));
}

#[test]
fn init_imports_hermes_sessions_into_resume_and_search() {
    let home = tempfile::tempdir().expect("temporary directory should be created");
    let data_dir = tempfile::tempdir().expect("temporary data directory should be created");
    let model_dir = tempfile::tempdir().expect("temporary model directory should be created");
    let hermes_session = home
        .path()
        .join(".hermes/sessions/session-1.jsonl");

    std::fs::create_dir_all(home.path().join(".hermes/sessions"))
        .expect("Hermes sessions directory should be created");
    std::fs::copy(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/hermes/minimal-session.jsonl"
        ),
        &hermes_session,
    )
    .expect("Hermes fixture should be copied");

    let init = std::process::Command::new(env!("CARGO_BIN_EXE_cli-memory"))
        .arg("init")
        .env("CLI_MEMORY_HOME", home.path())
        .env("CLI_MEMORY_DATA_DIR", data_dir.path())
        .env("CLI_MEMORY_MODEL_PATH", model_dir.path())
        .output()
        .expect("init should run");
    assert!(init.status.success());

    let hash_id = derive_resume_hash(ProviderKind::Hermes, "hermes-conv-1");
    let resume = std::process::Command::new(env!("CARGO_BIN_EXE_cli-memory"))
        .args(["resume", &hash_id])
        .env("CLI_MEMORY_HOME", home.path())
        .env("CLI_MEMORY_DATA_DIR", data_dir.path())
        .env("CLI_MEMORY_MODEL_PATH", model_dir.path())
        .output()
        .expect("resume should run");
    assert!(resume.status.success());
    let resume_stdout = String::from_utf8_lossy(&resume.stdout);
    assert!(resume_stdout.contains("user: How do I run the app?"));
    assert!(resume_stdout.contains("assistant: Use cargo run --bin cli-memory."));

    let search = std::process::Command::new(env!("CARGO_BIN_EXE_cli-memory"))
        .args(["conv-search", "run the app"])
        .env("CLI_MEMORY_HOME", home.path())
        .env("CLI_MEMORY_DATA_DIR", data_dir.path())
        .env("CLI_MEMORY_MODEL_PATH", model_dir.path())
        .output()
        .expect("conv-search should run");
    assert!(search.status.success());
    assert!(String::from_utf8_lossy(&search.stdout).contains("[hermes:hermes-conv-1]"));
}

#[test]
fn forget_bans_conversation_from_future_resume_and_search() {
    let home = tempfile::tempdir().expect("temporary directory should be created");
    let data_dir = tempfile::tempdir().expect("temporary data directory should be created");
    let model_dir = tempfile::tempdir().expect("temporary model directory should be created");
    let codex_session = home.path().join(".codex/sessions/session.jsonl");

    std::fs::create_dir_all(home.path().join(".codex/sessions"))
        .expect("Codex sessions directory should be created");
    std::fs::write(home.path().join(".codex/session_index.jsonl"), "{}\n")
        .expect("Codex session index should be written");
    std::fs::copy(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/codex/minimal-session.jsonl"
        ),
        &codex_session,
    )
    .expect("Codex fixture should be copied");

    let init = std::process::Command::new(env!("CARGO_BIN_EXE_cli-memory"))
        .arg("init")
        .env("CLI_MEMORY_HOME", home.path())
        .env("CLI_MEMORY_DATA_DIR", data_dir.path())
        .env("CLI_MEMORY_MODEL_PATH", model_dir.path())
        .output()
        .expect("init should run");
    assert!(init.status.success());

    let hash_id = derive_resume_hash(
        ProviderKind::Codex,
        "019e3c95-ac82-7402-bb65-f9bf46673f1f",
    );
    let forget = std::process::Command::new(env!("CARGO_BIN_EXE_cli-memory"))
        .args(["forget", "codex", &hash_id])
        .env("CLI_MEMORY_HOME", home.path())
        .env("CLI_MEMORY_DATA_DIR", data_dir.path())
        .output()
        .expect("forget should run");
    assert!(forget.status.success());
    assert!(String::from_utf8_lossy(&forget.stdout).contains("forgot codex"));

    let resume = std::process::Command::new(env!("CARGO_BIN_EXE_cli-memory"))
        .args(["resume", &hash_id])
        .env("CLI_MEMORY_HOME", home.path())
        .env("CLI_MEMORY_DATA_DIR", data_dir.path())
        .env("CLI_MEMORY_MODEL_PATH", model_dir.path())
        .output()
        .expect("resume should run");
    assert!(resume.status.success());
    assert!(String::from_utf8_lossy(&resume.stdout).contains("no conversation found"));

    let search = std::process::Command::new(env!("CARGO_BIN_EXE_cli-memory"))
        .args(["conv-search", "install"])
        .env("CLI_MEMORY_HOME", home.path())
        .env("CLI_MEMORY_DATA_DIR", data_dir.path())
        .env("CLI_MEMORY_MODEL_PATH", model_dir.path())
        .output()
        .expect("conv-search should run");
    assert!(search.status.success());
    assert!(String::from_utf8_lossy(&search.stdout).contains("no conversations matched"));

    let states = Storage::open(data_dir.path().join("db.sqlite3"))
        .expect("db should open")
        .conversation_states()
        .expect("conversation states should list");
    assert_eq!(states.len(), 1);
    assert_eq!(states[0].provider, ProviderKind::Codex);
    assert_eq!(
        states[0].conversation_id,
        "019e3c95-ac82-7402-bb65-f9bf46673f1f"
    );
    assert!(states[0].forgotten_at.is_some());
}

#[test]
fn doctor_prints_marker() {
    let home = tempfile::tempdir().expect("temporary directory should be created");
    let data_dir = tempfile::tempdir().expect("temporary data directory should be created");
    let model_dir = tempfile::tempdir().expect("temporary model directory should be created");
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_cli-memory"))
        .arg("doctor")
        .env("CLI_MEMORY_HOME", home.path())
        .env("CLI_MEMORY_DATA_DIR", data_dir.path())
        .env("CLI_MEMORY_MODEL_PATH", model_dir.path())
        .output()
        .expect("binary should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"db_ready\""));
    assert!(stdout.contains("\"detected_providers\""));
}

#[test]
fn stats_prints_json_counts() {
    let home = tempfile::tempdir().expect("temporary directory should be created");
    let data_dir = tempfile::tempdir().expect("temporary data directory should be created");
    let model_dir = tempfile::tempdir().expect("temporary model directory should be created");
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_cli-memory"))
        .arg("stats")
        .env("CLI_MEMORY_HOME", home.path())
        .env("CLI_MEMORY_DATA_DIR", data_dir.path())
        .env("CLI_MEMORY_MODEL_PATH", model_dir.path())
        .output()
        .expect("binary should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"total_conversations\""));
    assert!(stdout.contains("\"total_messages\""));
}

#[test]
fn init_detects_and_checkpoints_sources() {
    let home = tempfile::tempdir().expect("temporary directory should be created");
    let data_dir = tempfile::tempdir().expect("temporary data directory should be created");
    let model_dir = tempfile::tempdir().expect("temporary model directory should be created");
    let codex_session = home.path().join(".codex/sessions/session.jsonl");
    let claude_session = home.path().join(".claude/projects/project/session.jsonl");

    std::fs::create_dir_all(home.path().join(".claude/projects/project"))
        .expect("Claude projects directory should be created");
    std::fs::create_dir_all(home.path().join(".codex/sessions"))
        .expect("Codex sessions directory should be created");
    std::fs::write(home.path().join(".codex/session_index.jsonl"), "{}\n")
        .expect("Codex session index should be written");
    std::fs::copy(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/codex/minimal-session.jsonl"
        ),
        &codex_session,
    )
    .expect("Codex fixture should be copied");
    std::fs::copy(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/claude/minimal-session.jsonl"
        ),
        &claude_session,
    )
    .expect("Claude fixture should be copied");

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_cli-memory"))
        .arg("init")
        .env("CLI_MEMORY_HOME", home.path())
        .env("CLI_MEMORY_DATA_DIR", data_dir.path())
        .env("CLI_MEMORY_MODEL_PATH", model_dir.path())
        .output()
        .expect("binary should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("detected 2 providers"));
    assert!(stdout.contains("imported 2 conversations / 5 messages"));
    assert!(stdout.contains("claude"));
    assert!(stdout.contains("codex"));

    let storage = Storage::open(data_dir.path().join("db.sqlite3")).expect("db should open");
    let checkpoints = storage.list_checkpoints().expect("checkpoints should list");
    assert_eq!(checkpoints.len(), 3);
    assert_eq!(storage.count_conversations().expect("conversations should count"), 2);
    assert_eq!(storage.count_messages().expect("messages should count"), 5);
    assert_eq!(
        storage
            .count_message_embeddings()
            .expect("message embeddings should count"),
        5
    );
}

#[test]
fn refresh_updates_existing_checkpoint_without_duplicates() {
    let home = tempfile::tempdir().expect("temporary directory should be created");
    let data_dir = tempfile::tempdir().expect("temporary data directory should be created");
    let model_dir = tempfile::tempdir().expect("temporary model directory should be created");
    let codex_index = home.path().join(".codex/session_index.jsonl");
    let codex_session = home.path().join(".codex/sessions/session.jsonl");
    let claude_session = home.path().join(".claude/projects/project/session.jsonl");

    std::fs::create_dir_all(home.path().join(".claude/projects/project"))
        .expect("Claude projects directory should be created");
    std::fs::create_dir_all(home.path().join(".codex/sessions"))
        .expect("Codex sessions directory should be created");
    std::fs::write(&codex_index, "{}\n").expect("Codex session index should be written");
    std::fs::copy(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/codex/minimal-session.jsonl"
        ),
        &codex_session,
    )
    .expect("Codex fixture should be copied");
    std::fs::copy(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/claude/minimal-session.jsonl"
        ),
        &claude_session,
    )
    .expect("Claude fixture should be copied");

    let init = std::process::Command::new(env!("CARGO_BIN_EXE_cli-memory"))
        .arg("init")
        .env("CLI_MEMORY_HOME", home.path())
        .env("CLI_MEMORY_DATA_DIR", data_dir.path())
        .env("CLI_MEMORY_MODEL_PATH", model_dir.path())
        .output()
        .expect("init should run");
    assert!(init.status.success());

    let storage = Storage::open(data_dir.path().join("db.sqlite3")).expect("db should open");
    let before = storage.list_checkpoints().expect("checkpoints should list");
    let before_index = before
        .iter()
        .find(|row| row.source_path.ends_with("session_index.jsonl"))
        .expect("codex index checkpoint should exist")
        .fingerprint
        .clone();

    std::thread::sleep(std::time::Duration::from_secs(1));
    std::fs::write(&codex_index, "{\"changed\":true}\n")
        .expect("Codex session index should be updated");

    let refresh = std::process::Command::new(env!("CARGO_BIN_EXE_cli-memory"))
        .arg("refresh")
        .env("CLI_MEMORY_HOME", home.path())
        .env("CLI_MEMORY_DATA_DIR", data_dir.path())
        .env("CLI_MEMORY_MODEL_PATH", model_dir.path())
        .output()
        .expect("refresh should run");
    assert!(refresh.status.success());
    assert!(String::from_utf8_lossy(&refresh.stdout).contains("refreshed 2 providers"));

    let after = storage.list_checkpoints().expect("checkpoints should list");
    assert_eq!(after.len(), 3);
    let after_index = after
        .iter()
        .find(|row| row.source_path.ends_with("session_index.jsonl"))
        .expect("codex index checkpoint should exist")
        .fingerprint
        .clone();
    assert_ne!(before_index, after_index);
    assert_eq!(storage.count_conversations().expect("conversations should count"), 2);
    assert_eq!(storage.count_messages().expect("messages should count"), 5);
    assert_eq!(
        storage
            .count_message_embeddings()
            .expect("message embeddings should count"),
        5
    );
}

#[test]
fn refresh_imports_only_new_conversations_after_init() {
    let home = tempfile::tempdir().expect("temporary directory should be created");
    let data_dir = tempfile::tempdir().expect("temporary data directory should be created");
    let model_dir = tempfile::tempdir().expect("temporary model directory should be created");
    let codex_dir = home.path().join(".codex/sessions");
    let first_session = codex_dir.join("session-a.jsonl");
    let second_session = codex_dir.join("session-b.jsonl");

    std::fs::create_dir_all(&codex_dir).expect("Codex sessions directory should be created");
    std::fs::write(home.path().join(".codex/session_index.jsonl"), "{}\n")
        .expect("Codex session index should be written");
    std::fs::copy(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/codex/minimal-session.jsonl"
        ),
        &first_session,
    )
    .expect("Codex fixture should be copied");

    let init = std::process::Command::new(env!("CARGO_BIN_EXE_cli-memory"))
        .arg("init")
        .env("CLI_MEMORY_HOME", home.path())
        .env("CLI_MEMORY_DATA_DIR", data_dir.path())
        .env("CLI_MEMORY_MODEL_PATH", model_dir.path())
        .output()
        .expect("init should run");
    assert!(init.status.success());

    let second_text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/codex/minimal-session.jsonl"
    ))
    .expect("fixture should read")
    .replace(
        "019e3c95-ac82-7402-bb65-f9bf46673f1f",
        "019e3c95-ac82-7402-bb65-f9bf46673f2f",
    )
    .replace(
        "How do I install this?",
        "How do I configure this refresh test?",
    );
    std::fs::write(&second_session, second_text).expect("second session should be written");

    let refresh = std::process::Command::new(env!("CARGO_BIN_EXE_cli-memory"))
        .arg("refresh")
        .env("CLI_MEMORY_HOME", home.path())
        .env("CLI_MEMORY_DATA_DIR", data_dir.path())
        .env("CLI_MEMORY_MODEL_PATH", model_dir.path())
        .output()
        .expect("refresh should run");
    assert!(refresh.status.success());
    let stdout = String::from_utf8_lossy(&refresh.stdout);
    assert!(stdout.contains("imported 1 conversations / 2 messages"));

    let storage = Storage::open(data_dir.path().join("db.sqlite3")).expect("db should open");
    assert_eq!(storage.count_conversations().expect("conversations should count"), 2);
    assert_eq!(storage.count_messages().expect("messages should count"), 4);
    assert_eq!(
        storage
            .count_message_embeddings()
            .expect("message embeddings should count"),
        4
    );
}

#[test]
fn refresh_recovers_from_checkpointed_but_unimported_copilot_sources() {
    let home = tempfile::tempdir().expect("temporary directory should be created");
    let data_dir = tempfile::tempdir().expect("temporary data directory should be created");
    let model_dir = tempfile::tempdir().expect("temporary model directory should be created");
    let copilot_dir = home.path().join(".copilot/session-state/copilot-a");
    let session = copilot_dir.join("events.jsonl");

    std::fs::create_dir_all(&copilot_dir).expect("Copilot session directory should be created");
    std::fs::copy(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/copilot/real-events-session.jsonl"
        ),
        &session,
    )
    .expect("Copilot fixture should be copied");

    let storage = Storage::open(data_dir.path().join("db.sqlite3")).expect("db should open");
    storage
        .save_checkpoint(&Checkpoint {
            provider: ProviderKind::Copilot,
            source_path: session.display().to_string(),
            fingerprint: "file:281:123456".to_owned(),
            updated_at: Utc::now(),
        })
        .expect("checkpoint should save");

    let metadata = std::fs::metadata(&session).expect("session metadata should read");
    let modified = metadata
        .modified()
        .expect("modified time should exist")
        .duration_since(std::time::UNIX_EPOCH)
        .expect("modified time should be after epoch")
        .as_secs();
    storage
        .save_checkpoint(&Checkpoint {
            provider: ProviderKind::Copilot,
            source_path: session.display().to_string(),
            fingerprint: format!("file:{}:{modified}", metadata.len()),
            updated_at: Utc::now(),
        })
        .expect("matching checkpoint should save");

    let refresh = std::process::Command::new(env!("CARGO_BIN_EXE_cli-memory"))
        .arg("refresh")
        .env("CLI_MEMORY_HOME", home.path())
        .env("CLI_MEMORY_DATA_DIR", data_dir.path())
        .env("CLI_MEMORY_MODEL_PATH", model_dir.path())
        .output()
        .expect("refresh should run");
    assert!(refresh.status.success());
    let stdout = String::from_utf8_lossy(&refresh.stdout);
    assert!(stdout.contains("imported 1 conversations / 2 messages"));

    assert_eq!(storage.count_conversations().expect("conversations should count"), 1);
    assert_eq!(storage.count_messages().expect("messages should count"), 2);
}

#[test]
fn refresh_skips_reimporting_changed_existing_conversations() {
    let home = tempfile::tempdir().expect("temporary directory should be created");
    let data_dir = tempfile::tempdir().expect("temporary data directory should be created");
    let model_dir = tempfile::tempdir().expect("temporary model directory should be created");
    let codex_dir = home.path().join(".codex/sessions");
    let session = codex_dir.join("session-a.jsonl");

    std::fs::create_dir_all(&codex_dir).expect("Codex sessions directory should be created");
    std::fs::write(home.path().join(".codex/session_index.jsonl"), "{}\n")
        .expect("Codex session index should be written");
    std::fs::copy(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/codex/minimal-session.jsonl"
        ),
        &session,
    )
    .expect("Codex fixture should be copied");

    let init = std::process::Command::new(env!("CARGO_BIN_EXE_cli-memory"))
        .arg("init")
        .env("CLI_MEMORY_HOME", home.path())
        .env("CLI_MEMORY_DATA_DIR", data_dir.path())
        .env("CLI_MEMORY_MODEL_PATH", model_dir.path())
        .output()
        .expect("init should run");
    assert!(init.status.success());

    std::thread::sleep(std::time::Duration::from_secs(1));
    let mut changed_text = std::fs::read_to_string(&session).expect("session should read");
    changed_text.push_str(
        "{\"timestamp\":\"2026-05-18T19:35:34.000Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"role\":\"assistant\",\"content\":[{\"type\":\"output_text\",\"text\":\"Append one more line that should not trigger reimport.\"}]}}\n",
    );
    std::fs::write(&session, changed_text).expect("session should be updated");

    let refresh = std::process::Command::new(env!("CARGO_BIN_EXE_cli-memory"))
        .arg("refresh")
        .env("CLI_MEMORY_HOME", home.path())
        .env("CLI_MEMORY_DATA_DIR", data_dir.path())
        .env("CLI_MEMORY_MODEL_PATH", model_dir.path())
        .output()
        .expect("refresh should run");
    assert!(refresh.status.success());
    let stdout = String::from_utf8_lossy(&refresh.stdout);
    assert!(stdout.contains("imported 0 conversations / 0 messages"));

    let storage = Storage::open(data_dir.path().join("db.sqlite3")).expect("db should open");
    assert_eq!(storage.count_conversations().expect("conversations should count"), 1);
    assert_eq!(storage.count_messages().expect("messages should count"), 2);
    assert_eq!(
        storage
            .count_message_embeddings()
            .expect("message embeddings should count"),
        2
    );
}
