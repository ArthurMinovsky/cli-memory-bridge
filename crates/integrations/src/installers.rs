pub fn render_codex_install(binary_path: &str) -> String {
    format!(
        "[mcp_servers.cli-memory]\ncommand = \"{binary_path}\"\nargs = [\"serve\"]\nstartup_timeout_sec = 120\n"
    )
}

pub fn render_zed_install(binary_path: &str) -> String {
    format!(
        "{{\"context_servers\":{{\"cli-memory\":{{\"command\":\"{binary_path}\",\"args\":[\"serve\"]}}}}}}"
    )
}

pub fn render_hermes_install(binary_path: &str) -> String {
    format!(
        "mcp_servers:\n  cli-memory:\n    command: \"{binary_path}\"\n    args:\n      - \"serve\"\n"
    )
}

pub fn render_gemini_install(binary_path: &str) -> String {
    format!(
        "{{\"mcpServers\":{{\"cli-memory\":{{\"command\":\"{binary_path}\",\"args\":[\"serve\"]}}}}}}"
    )
}

pub fn render_opencode_install(binary_path: &str) -> String {
    format!(
        "{{\"mcp\":{{\"cli-memory\":{{\"type\":\"local\",\"command\":[\"{binary_path}\",\"serve\"],\"enabled\":true}}}}}}"
    )
}

pub fn render_copilot_install(binary_path: &str) -> String {
    format!(
        "{{\"mcpServers\":{{\"cli-memory\":{{\"type\":\"local\",\"command\":\"{binary_path}\",\"args\":[\"serve\"],\"tools\":[\"*\"]}}}}}}"
    )
}
