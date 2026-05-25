use std::path::{Path, PathBuf};

use anyhow::Result;
use cli_memory_core::ProviderKind;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DetectedProvider {
    pub provider: ProviderKind,
    pub paths: Vec<PathBuf>,
}

pub fn detect_providers(home: &Path) -> Result<Vec<DetectedProvider>> {
    let mut detected = Vec::new();

    let claude_projects = home.join(".claude/projects");
    if claude_projects.exists() {
        detected.push(DetectedProvider {
            provider: ProviderKind::Claude,
            paths: vec![claude_projects],
        });
    }

    let codex_sessions = home.join(".codex/sessions");
    let codex_index = home.join(".codex/session_index.jsonl");
    if codex_sessions.exists() || codex_index.exists() {
        let mut paths = Vec::new();
        if codex_sessions.exists() {
            paths.push(codex_sessions);
        }
        if codex_index.exists() {
            paths.push(codex_index);
        }
        detected.push(DetectedProvider {
            provider: ProviderKind::Codex,
            paths,
        });
    }

    let gemini_tmp = home.join(".gemini/tmp");
    if gemini_tmp.exists() {
        detected.push(DetectedProvider {
            provider: ProviderKind::Gemini,
            paths: vec![gemini_tmp],
        });
    }

    let copilot_state = home.join(".copilot/session-state");
    if copilot_state.exists() {
        detected.push(DetectedProvider {
            provider: ProviderKind::Copilot,
            paths: vec![copilot_state],
        });
    }

    let zed_conversations = home.join(".config/zed/conversations");
    if zed_conversations.exists() {
        detected.push(DetectedProvider {
            provider: ProviderKind::Zed,
            paths: vec![zed_conversations],
        });
    }

    let opencode_sessions = home.join(".local/share/opencode/storage/session_diff");
    let opencode_db = home.join(".local/share/opencode/opencode.db");
    if opencode_sessions.exists() || opencode_db.exists() {
        let mut paths = Vec::new();
        if opencode_sessions.exists() {
            paths.push(opencode_sessions);
        }
        if opencode_db.exists() {
            paths.push(opencode_db);
        }
        detected.push(DetectedProvider {
            provider: ProviderKind::OpenCode,
            paths,
        });
    }

    let hermes_sessions = home.join(".hermes/sessions");
    let hermes_history = home.join(".hermes/history");
    if hermes_sessions.exists() || hermes_history.exists() {
        let mut paths = Vec::new();
        if hermes_sessions.exists() {
            paths.push(hermes_sessions);
        }
        if hermes_history.exists() {
            paths.push(hermes_history);
        }
        detected.push(DetectedProvider {
            provider: ProviderKind::Hermes,
            paths,
        });
    }

    let antigravity_brain = home.join(".gemini/antigravity/brain");
    if antigravity_brain.exists() {
        detected.push(DetectedProvider {
            provider: ProviderKind::AntigravityCli,
            paths: vec![antigravity_brain],
        });
    }

    Ok(detected)
}
