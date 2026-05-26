#!/usr/bin/env node

const fs = require("fs");
const path = require("path");
// Use async spawn so stdin/stdout are streamed in real-time.
// spawnSync buffers all I/O until the child exits, which breaks the MCP
// JSON-RPC handshake for the long-running `serve` subcommand.
const { spawn } = require("child_process");
const { binaryName, platformKey } = require("../platform");

function candidateBinaries() {
  const name = binaryName(process.platform);
  const root = path.resolve(__dirname, "..", "..");
  const currentPlatformKey = platformKey(process.platform, process.arch);

  const candidates = [];

  if (process.env.CLI_MEMORY_BINARY) {
    candidates.push(process.env.CLI_MEMORY_BINARY);
  }

  candidates.push(
    path.join(root, "npm", "vendor", currentPlatformKey, name),
    path.join(root, "target", "release", name),
    path.join(root, "target", "debug", name)
  );

  return candidates;
}

function resolveBinary() {
  for (const candidate of candidateBinaries()) {
    if (candidate && fs.existsSync(candidate)) {
      return candidate;
    }
  }

  return null;
}

const binary = resolveBinary();

if (!binary) {
  const currentPlatformKey = platformKey(process.platform, process.arch);
  console.error(
    [
      "cli-memory: no bundled binary was found for this package.",
      `expected platform key: ${currentPlatformKey}`,
      "checked npm/vendor, target/release, and target/debug",
      "set CLI_MEMORY_BINARY to an explicit executable path to override",
    ].join("\n")
  );
  process.exit(1);
}

const child = spawn(binary, process.argv.slice(2), {
  stdio: "inherit", // stream stdin/stdout/stderr directly — no buffering
});

child.on("error", (err) => {
  console.error(`cli-memory: failed to launch ${binary}`);
  console.error(err.message);
  process.exit(1);
});

// Forward signals so Ctrl-C / SIGTERM reach the child correctly.
for (const sig of ["SIGINT", "SIGTERM", "SIGHUP"]) {
  process.on(sig, () => child.kill(sig));
}

child.on("exit", (code, signal) => {
  if (signal) {
    process.kill(process.pid, signal);
  } else {
    process.exit(code ?? 1);
  }
});
