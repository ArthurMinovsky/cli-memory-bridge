pub fn render_codex_resume_skill() -> String {
    "$resume\nUse the cli-memory MCP tool `resume` with the provided hash id."
        .to_owned()
}

pub fn render_codex_conv_search_skill() -> String {
    "$conv-search\nUse the cli-memory MCP tool `conv-search` with the provided query."
        .to_owned()
}

pub fn render_codex_forget_skill() -> String {
    "$forget\nUse the cli-memory forget flow with both provider and hash id, for example `cmb forget codex <hash-id>`."
        .to_owned()
}

pub fn render_claude_resume_command() -> String {
    "/resume\nUse the cli-memory MCP tool `resume` with the provided hash id."
        .to_owned()
}

pub fn render_claude_conv_search_command() -> String {
    "/conv-search\nUse the cli-memory MCP tool `conv-search` with the provided query."
        .to_owned()
}

pub fn render_claude_forget_command() -> String {
    "/forget\nUse the cli-memory forget flow with both provider and hash id, for example `cmb forget claude <hash-id>`."
        .to_owned()
}
