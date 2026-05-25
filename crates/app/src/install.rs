use anyhow::Result;
use cli_memory_core::ProviderKind;
use serde_json::{Value, json};

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
            "mode": "assets",
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
            "config_path": "~/.config/github-copilot-cli/config.json",
            "preferred_launcher": "npx",
            "npx_snippet": json!({
                "mcpServers": {
                    "cli-memory": {
                        "type": "local",
                        "command": "npx",
                        "args": ["-y", "@aminovsky/cli-memory", "serve"],
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

pub fn render_unlink_bundle(provider: ProviderKind) -> Result<Value> {
    let value = match provider {
        ProviderKind::Codex => json!({
            "provider": provider.as_slug(),
            "mode": "assets",
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
            "config_path": "~/.config/github-copilot-cli/config.json",
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
