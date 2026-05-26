---
name: cli-memory-retrieval
description: Retrieve context from cli-memory MCP tools and imported local transcript sources. Use when resuming conversations, searching imported CLI history, investigating missing resume ids or hash ids, or reading transcript context from Codex, Claude, Gemini, Copilot, Zed, Hermes, OpenCode, or Antigravity CLI. Always use `rtk rg`, `rtk cat`, or narrow `rtk sed` for local file reads during retrieval; do not use the native Read tool for transcript/context retrieval.
---

# cli-memory-retrieval

Use this workflow when retrieving context from `cli-memory` or from raw imported transcript sources.

## Rules

- Use `cli-memory` MCP or CLI first for `resume`, `conv-search`, `doctor`, and `stats`.
- When a resume id or hash id is provided, prefer exact-match lookup before broader semantic search.
- For local file reads, use only:
  - `rtk rg`
  - `rtk cat`
  - narrow `rtk sed -n`
- Do not use the native Read tool for transcript or context retrieval.

## Retrieval order

1. Try `cli-memory resume <id>` or the matching MCP `resume` tool.
2. If resume fails, use exact-match source lookup with `rtk rg -n "<id>" <known paths>`.
3. Read only the matched files with `rtk cat` or narrow `rtk sed -n`.
4. Use `conv-search` only after exact-match lookup if the user already provided a concrete id.

## Output

- Prefer quoting the exact matched source path when a resume miss falls back to raw sources.
- Keep retrieval evidence provider-scoped when possible.
