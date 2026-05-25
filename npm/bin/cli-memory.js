#!/usr/bin/env node

const fs = require("fs");
const path = require("path");
const { spawnSync } = require("child_process");
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

const result = spawnSync(binary, process.argv.slice(2), {
  stdio: "inherit",
});

if (result.error) {
  console.error(`cli-memory: failed to launch ${binary}`);
  console.error(result.error.message);
  process.exit(1);
}

process.exit(result.status ?? 1);
