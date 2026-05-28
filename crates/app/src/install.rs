use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use cli_memory_core::ProviderKind;
use cli_memory_integrations::DetectedProvider;
use serde_json::{json, Value};

pub fn all_providers() -> &'static [ProviderKind] {
    &[
        ProviderKind::Codex,
        ProviderKind::Claude,
        ProviderKind::Gemini,
        ProviderKind::OpenCode,
        ProviderKind::Copilot,
        ProviderKind::Hermes,
        ProviderKind::Zed,
        ProviderKind::AntigravityCli,
    ]
}

pub fn render_install_bundle(provider: ProviderKind, binary_path: &str) -> Result<Value> {
    let npx_json = json!({
        "command": "npx",
        "args": ["-y", "@aminovsky/cli-memory", "serve"],
    });

    let value = match provider {
        ProviderKind::Codex => json!({
            "provider": provider.as_slug(),
            "mode": "config+assets",
            "config_path": "~/.codex/config.toml",
            "preferred_launcher": "binary-path",
            "binary_snippet": cli_memory_integrations::render_codex_install(binary_path),
            "assets": {
                "$resume": cli_memory_integrations::render_codex_resume_skill(),
                "$conv-search": cli_memory_integrations::render_codex_conv_search_skill(),
                "$forget": cli_memory_integrations::render_codex_forget_skill(),
            },
        }),
        ProviderKind::Claude => json!({
            "provider": provider.as_slug(),
            "mode": "assets",
            "assets": {
                "/resume": cli_memory_integrations::render_claude_resume_command(),
                "/conv-search": cli_memory_integrations::render_claude_conv_search_command(),
                "/forget": cli_memory_integrations::render_claude_forget_command(),
            },
        }),
        ProviderKind::Hermes => json!({
            "provider": provider.as_slug(),
            "mode": "config",
            "config_path": "~/.hermes/config.yaml",
            "preferred_launcher": "npx",
            "npx_snippet": "mcp_servers:\n  cli-memory:\n    command: \"npx\"\n    args:\n      - \"-y\"\n      - \"@aminovsky/cli-memory\"\n      - \"serve\"\n",
            "binary_snippet": cli_memory_integrations::render_hermes_install(binary_path),
        }),
        ProviderKind::Gemini => json!({
            "provider": provider.as_slug(),
            "mode": "config",
            "config_path": "~/.gemini/settings.json",
            "preferred_launcher": "npx",
            "npx_snippet": json!({
                "mcpServers": {
                    "cli-memory": npx_json,
                }
            }).to_string(),
            "binary_snippet": cli_memory_integrations::render_gemini_install(binary_path),
        }),
        ProviderKind::OpenCode => json!({
            "provider": provider.as_slug(),
            "mode": "config",
            "config_path": "~/.config/opencode/config.json",
            "preferred_launcher": "npx",
            "npx_snippet": json!({
                "mcp": {
                    "cli-memory": {
                        "type": "local",
                        "command": ["npx", "-y", "@aminovsky/cli-memory", "serve"],
                        "enabled": true,
                    }
                }
            }).to_string(),
            "binary_snippet": cli_memory_integrations::render_opencode_install(binary_path),
        }),
        ProviderKind::Copilot => json!({
            "provider": provider.as_slug(),
            "mode": "config",
            "config_path": "~/.copilot/mcp-config.json",
            "preferred_launcher": "global-command",
            "global_command_snippet": json!({
                "mcpServers": {
                    "cli-memory": {
                        "type": "local",
                        "command": "cli-memory",
                        "args": ["serve"],
                        "tools": ["*"],
                    }
                }
            }).to_string(),
            "binary_snippet": cli_memory_integrations::render_copilot_install(binary_path),
        }),
        ProviderKind::Zed => json!({
            "provider": provider.as_slug(),
            "mode": "config",
            "config_path": "~/.config/zed/settings.json",
            "preferred_launcher": "npx",
            "npx_snippet": json!({
                "context_servers": {
                    "cli-memory": npx_json,
                }
            }).to_string(),
            "binary_snippet": cli_memory_integrations::render_zed_install(binary_path),
        }),
        ProviderKind::AntigravityCli => json!({
            "provider": provider.as_slug(),
            "mode": "config",
            "config_path": "~/.gemini/antigravity/mcp_config.json",
            "preferred_launcher": "npx",
            "npx_snippet": json!({
                "mcpServers": {
                    "cli-memory": {
                        "command": "npx",
                        "args": ["-y", "@aminovsky/cli-memory", "serve"],
                    }
                }
            }).to_string(),
        }),
    };

    Ok(value)
}

pub fn render_install_all_bundle(binary_path: &str) -> Result<Value> {
    Ok(json!({
        "mode": "install-all",
        "providers": all_providers()
            .iter()
            .map(|provider| render_install_bundle(*provider, binary_path))
            .collect::<Result<Vec<_>>>()?,
    }))
}

pub fn render_unlink_bundle(provider: ProviderKind) -> Result<Value> {
    let value = match provider {
        ProviderKind::Codex => json!({
            "provider": provider.as_slug(),
            "mode": "config+assets",
            "config_path": "~/.codex/config.toml",
            "remove_keys": ["mcp_servers.cli-memory"],
            "remove_assets": ["$resume", "$conv-search", "$forget"],
            "notes": "Remove the cli-memory Codex skill or plugin-owned command assets from your Codex setup.",
        }),
        ProviderKind::Claude => json!({
            "provider": provider.as_slug(),
            "mode": "assets",
            "remove_assets": ["/resume", "/conv-search", "/forget"],
            "notes": "Remove the cli-memory Claude command assets from your Claude Code setup.",
        }),
        ProviderKind::Hermes => json!({
            "provider": provider.as_slug(),
            "mode": "config",
            "config_path": "~/.hermes/config.yaml",
            "remove_keys": ["mcp_servers.cli-memory"],
        }),
        ProviderKind::Gemini => json!({
            "provider": provider.as_slug(),
            "mode": "config",
            "config_path": "~/.gemini/settings.json",
            "remove_keys": ["mcpServers.cli-memory"],
        }),
        ProviderKind::OpenCode => json!({
            "provider": provider.as_slug(),
            "mode": "config",
            "config_path": "~/.config/opencode/config.json",
            "remove_keys": ["mcp.cli-memory"],
        }),
        ProviderKind::Copilot => json!({
            "provider": provider.as_slug(),
            "mode": "config",
            "config_path": "~/.copilot/mcp-config.json",
            "remove_keys": ["mcpServers.cli-memory"],
        }),
        ProviderKind::Zed => json!({
            "provider": provider.as_slug(),
            "mode": "config",
            "config_path": "~/.config/zed/settings.json",
            "remove_keys": ["context_servers.cli-memory"],
        }),
        ProviderKind::AntigravityCli => json!({
            "provider": provider.as_slug(),
            "mode": "config",
            "config_path": "~/.gemini/antigravity/mcp_config.json",
            "remove_keys": ["mcpServers.cli-memory"],
        }),
    };

    Ok(value)
}

pub fn render_unlink_all_bundle() -> Result<Value> {
    Ok(json!({
        "mode": "unlink-all",
        "providers": all_providers()
            .iter()
            .map(|provider| render_unlink_bundle(*provider))
            .collect::<Result<Vec<_>>>()?,
    }))
}

pub fn render_uninstall_bundle() -> Result<Value> {
    Ok(json!({
        "mode": "uninstall",
        "steps": [
            "Run `cli-memory unlink --all` first to remove provider MCP/config links.",
            "If you installed the npm package globally, remove it with `npm uninstall -g @aminovsky/cli-memory`.",
            "If you only use `npx -y @aminovsky/cli-memory ...`, there is no global package to remove.",
        ],
        "unlink_all_command": "cli-memory unlink --all",
        "npm_uninstall_command": "npm uninstall -g @aminovsky/cli-memory",
    }))
}

pub struct IntegrationInstallSummary {
    pub installed: Vec<String>,
    pub skipped: Vec<String>,
}

pub fn ensure_detected_integrations(
    home: &Path,
    detected: &[DetectedProvider],
    binary_path: &str,
) -> Result<IntegrationInstallSummary> {
    let mut installed = Vec::new();
    let mut skipped = Vec::new();

    for provider in detected {
        let changed = match provider.provider {
            ProviderKind::Codex => ensure_toml_snippet(
                &home.join(".codex/config.toml"),
                "[mcp_servers.cli-memory]",
                &cli_memory_integrations::render_codex_install(binary_path),
            )?,
            ProviderKind::Claude => ensure_json_entry(
                &home.join(".claude/settings.json"),
                &["mcpServers", "cli-memory"],
                json!({
                    "command": binary_path,
                    "args": ["serve"],
                }),
            )?,
            ProviderKind::Gemini => ensure_json_entry(
                &preferred_existing_json_path(
                    &[
                        home.join(".gemini/settings.json"),
                        home.join(".gemini/config/mcp_config.json"),
                    ],
                    home.join(".gemini/settings.json"),
                ),
                &["mcpServers", "cli-memory"],
                json!({
                    "command": binary_path,
                    "args": ["serve"],
                }),
            )?,
            ProviderKind::OpenCode => ensure_json_entry(
                &home.join(".config/opencode/opencode.json"),
                &["mcp", "cli-memory"],
                json!({
                    "type": "local",
                    "command": [binary_path, "serve"],
                    "enabled": true,
                }),
            )?,
            ProviderKind::Copilot => ensure_json_entry(
                &home.join(".copilot/mcp-config.json"),
                &["mcpServers", "cli-memory"],
                json!({
                    "type": "local",
                    "command": binary_path,
                    "args": ["serve"],
                    "tools": ["*"],
                }),
            )?,
            ProviderKind::Hermes => ensure_toml_snippet(
                &home.join(".hermes/config.yaml"),
                "cli-memory:",
                &cli_memory_integrations::render_hermes_install(binary_path),
            )?,
            ProviderKind::Zed => ensure_json_entry(
                &home.join(".config/zed/settings.json"),
                &["context_servers", "cli-memory"],
                json!({
                    "enabled": true,
                    "remote": false,
                    "command": binary_path,
                    "args": ["serve"],
                }),
            )?,
            ProviderKind::AntigravityCli => ensure_json_entry(
                &home.join(".gemini/antigravity-cli/settings.json"),
                &["mcpServers", "cli-memory"],
                json!({
                    "command": binary_path,
                    "args": ["serve"],
                }),
            )?,
        };

        if changed {
            installed.push(provider.provider.as_slug().to_owned());
        } else {
            skipped.push(provider.provider.as_slug().to_owned());
        }
    }

    Ok(IntegrationInstallSummary { installed, skipped })
}

fn preferred_existing_json_path(candidates: &[PathBuf], fallback: PathBuf) -> PathBuf {
    candidates
        .iter()
        .find(|path| path.exists())
        .cloned()
        .unwrap_or(fallback)
}

fn ensure_toml_snippet(path: &Path, marker: &str, snippet: &str) -> Result<bool> {
    let existing = fs::read_to_string(path).unwrap_or_default();
    if existing.contains(marker) {
        return Ok(false);
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create config directory {}", parent.display()))?;
    }

    let mut updated = existing;
    if !updated.is_empty() && !updated.ends_with('\n') {
        updated.push('\n');
    }
    updated.push_str(snippet);
    fs::write(path, updated)
        .with_context(|| format!("failed to write config {}", path.display()))?;
    Ok(true)
}

fn strip_jsonc_comments(raw: &str) -> String {
    raw.lines()
        .filter(|line| {
            let trimmed = line.trim_start();
            !trimmed.starts_with("//")
        })
        .map(|line| {
            // Remove inline // comments, but be cautious about strings
            // Simple heuristic: strip everything after // that is not inside quotes
            if let Some(pos) = line.find("//") {
                let before = &line[..pos];
                // Only strip if // is outside of strings (even count of unescaped " before it)
                let quote_count = before.chars().filter(|&c| c == '"').count();
                if quote_count % 2 == 0 {
                    before.to_string()
                } else {
                    line.to_string()
                }
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        // Also remove trailing commas before ] or }
        .replace(",]", "]")
        .replace(",}", "}")
}

fn ensure_json_entry(path: &Path, key_path: &[&str], entry: Value) -> Result<bool> {
    let mut root = if path.exists() {
        let raw = fs::read_to_string(path)
            .with_context(|| format!("failed to read config {}", path.display()))?;
        let cleaned = strip_jsonc_comments(&raw);
        serde_json::from_str::<Value>(&cleaned)
            .with_context(|| format!("failed to parse json config {}", path.display()))?
    } else {
        json!({})
    };

    let mut cursor = &mut root;
    for key in &key_path[..key_path.len() - 1] {
        if !cursor.is_object() {
            *cursor = json!({});
        }
        let object = cursor.as_object_mut().expect("json object should exist");
        cursor = object.entry((*key).to_owned()).or_insert_with(|| json!({}));
    }

    if !cursor.is_object() {
        *cursor = json!({});
    }
    let object = cursor.as_object_mut().expect("json object should exist");
    let leaf = key_path[key_path.len() - 1];
    if object.contains_key(leaf) {
        return Ok(false);
    }
    object.insert(leaf.to_owned(), entry);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create config directory {}", parent.display()))?;
    }
    fs::write(
        path,
        serde_json::to_string_pretty(&root).expect("json config should serialize"),
    )
    .with_context(|| format!("failed to write config {}", path.display()))?;
    Ok(true)
}
