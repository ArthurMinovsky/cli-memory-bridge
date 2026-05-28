---
name: cli-memory-retrieval
description: Retrieve context from cli-memory MCP tools and imported local transcript sources. Use when resuming conversations, searching imported CLI history, investigating missing resume ids or hash ids, or reading transcript context from Codex, Claude, Gemini, Copilot, Zed, Hermes, OpenCode, or Antigravity CLI. Always use `rtk` commands to extract key content with minimal redundant tokens; do not use the native Read tool for transcript/context retrieval.
---

# cli-memory-retrieval

Use this workflow when retrieving context from `cli-memory` or from raw imported transcript sources.

## Rules

- Use `cli-memory` MCP or the local `cmb` CLI first for `resume`, `conv-search`, `doctor`, and `stats`.
- Use `rtk` as a source-inspection layer, not as a replacement for the memory index.
- When a resume id or hash id is provided, prefer exact-match lookup before broader semantic search.
- For local file reads, use this order:
  - `rtk rg -n` to find exact matches and line numbers
  - narrow `rtk sed -n` around the exact match
  - `rtk cat` only when the whole matched file is already small enough
- Do not use the native Read tool for transcript or context retrieval.
- Do not `cat` large transcript files end-to-end when `rtk rg` or narrow `rtk sed -n` can extract the key content.

## Retrieval order

1. Try `cmb resume <id>` or the matching MCP `resume` tool.
2. If resume fails, use exact-match source lookup with `rtk rg -n "<id>" <known paths>`.
3. Inspect only the matched slices with narrow `rtk sed -n` first.
4. Use `rtk cat` only for small matched files or when the whole file is required.
5. Use `conv-search` only after exact-match lookup if the user already provided a concrete id.

## Output

- Prefer quoting the exact matched source path when a resume miss falls back to raw sources.
- Keep retrieval evidence provider-scoped when possible.
- If `cli-memory` or another tool emits a large saved-output file, inspect that file with `rtk rg` and narrow `rtk sed -n` instead of full-file reads.
