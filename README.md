# cli-memory-bridge

`cli-memory-bridge` is a local-first memory bridge for terminal coding agents and MCP-enabled CLIs. It imports conversation history from supported providers on your machine, stores canonical transcripts and embeddings locally, and serves that memory back through `cmb` commands and an MCP-compatible stdio server.

The current Rust build supports transcript import and retrieval for:
- Codex
- Claude
- Gemini
- Copilot
- Zed
- OpenCode
- Hermes
- Antigravity CLI

It also ships an npm package surface so MCP configs can launch it through:

```bash
npx -y @aminovsky/cli-memory serve
```

## Features

- `cmb init` to detect providers and import local conversations
- `cmb refresh` to incrementally import only new or changed sources
- `cmb resume <hash-id>` to restore a conversation transcript by stable hash
- `cmb conv-search <query>` to search imported conversation content
- `cmb forget <provider> <hash-id>` to soft-ban a conversation from future retrieval
- `cmb serve` to expose the stdio MCP tool surface
- `cmb doctor` and `cmb stats` for diagnostics
- `cmb install <provider>` to render provider install/config bundles
- `cmb unlink <provider>` and `cmb unlink --all` to render provider cleanup bundles
- `cmb uninstall` to render full package removal guidance

## Install

### npm / npx

For MCP usage and command execution through npm:

```bash
npx -y @aminovsky/cli-memory --help
```

On first install, the npm package downloads a platform-specific `cmb` binary from the matching GitHub release for this package version and stores it under `npm/vendor/<platform>/`.

If you install the package globally:

```bash
npm install -g @aminovsky/cli-memory
cmb --help
```

### Local Rust build

```bash
cargo build --release
./target/release/cmb --help
```

## Quick start

Import local transcripts and start using retrieval:

```bash
cmb init
cmb doctor
cmb stats
cmb conv-search "run the app"
```

Run the MCP server through `npx`:

```bash
npx -y @aminovsky/cli-memory serve
```

## MCP launch

The npm package is designed for MCP configs that accept `command` plus `args`. The preferred launcher shape is:

```json
{
  "command": "npx",
  "args": ["-y", "@aminovsky/cli-memory", "serve"]
}
```

## Provider install

Render a provider-specific install bundle:

```bash
cmb install gemini
cmb install zed
cmb install codex
```

Current install behavior is render-only. It prints the config/assets you should apply for that provider.

## Provider unlink

Render cleanup instructions for one provider:

```bash
cmb unlink gemini
cmb unlink zed
cmb unlink codex
```

Render cleanup bundles for every supported provider:

```bash
cmb unlink --all
```

`unlink` removes provider wiring only. It does not remove the `cli-memory` package or the local memory database.

## Full uninstall

Render full uninstall guidance:

```bash
cmb uninstall
```

The intended order is:

```bash
cmb unlink --all
npm uninstall -g @aminovsky/cli-memory
```

If you only use `npx -y @aminovsky/cli-memory ...`, there may be no global npm package to remove.

## Supported local transcript sources

- Codex: `~/.codex/sessions`, `~/.codex/session_index.jsonl`
- Claude: `~/.claude/projects`
- Gemini: `~/.gemini/tmp/*/chats/session-*`
- Copilot: `~/.copilot/session-state/*/events.jsonl`
- Zed: `~/.config/zed/conversations/*.zed.json`
- OpenCode: `~/.local/share/opencode/storage/session_diff/*.json` with `opencode.db` fallback
- Hermes: `~/.hermes/sessions` / `~/.hermes/history`
- Antigravity CLI: `~/.gemini/antigravity/brain/*`

## Release notes

For npm release prep, this repo now includes:
- package metadata in [package.json](/Users/aminovsky/Desktop/cli-memory-bridge-rs/package.json)
- Apache 2.0 license in [LICENSE](/Users/aminovsky/Desktop/cli-memory-bridge-rs/LICENSE)
- npm bin entrypoint in [npm/bin/cli-memory.js](/Users/aminovsky/Desktop/cli-memory-bridge-rs/npm/bin/cli-memory.js)
- npm postinstall downloader in [npm/install.js](/Users/aminovsky/Desktop/cli-memory-bridge-rs/npm/install.js)
- shared platform mapping in [npm/platform.js](/Users/aminovsky/Desktop/cli-memory-bridge-rs/npm/platform.js)

Before publishing to npm:
- build release binaries for each supported platform
- upload them to the GitHub release tagged `v<package.json version>`
- use asset names in the form `cmb-<version>-<platform-arch>` and `cmb.exe-<version>-<platform-arch>` for Windows
- verify with `npm pack --dry-run` and a clean `npx -y @aminovsky/cli-memory --help` install test

## Status

This build is production-oriented but still evolving:
- transcript import and retrieval are implemented
- provider lifecycle commands are implemented
- some providers are reconstructed from artifact trails rather than full native session models
- graph/knowledge-linking is intentionally deferred

## Special Thanks

This project takes reference and inspiration from:
- [rohitg00/agentmemory](https://github.com/rohitg00/agentmemory)
- [aiming-lab/SimpleMem](https://github.com/aiming-lab/SimpleMem)
- [RyanCodrai/turbovec](https://github.com/RyanCodrai/turbovec)
