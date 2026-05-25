"use strict";

const fs = require("fs");
const https = require("https");
const path = require("path");

const {
  binaryName,
  platformKey,
  releaseAssetName,
  supportedPlatform,
} = require("./platform");

function repoRoot() {
  return path.resolve(__dirname, "..");
}

function packageVersion() {
  const packageJson = JSON.parse(
    fs.readFileSync(path.join(repoRoot(), "package.json"), "utf8")
  );
  return packageJson.version;
}

function vendorDir() {
  return path.join(repoRoot(), "npm", "vendor", platformKey(process.platform, process.arch));
}

function vendorBinaryPath() {
  return path.join(vendorDir(), binaryName(process.platform));
}

function releaseUrl(version) {
  const asset = releaseAssetName(version, process.platform, process.arch);
  return `https://github.com/ArthurMinovsky/cli-memory-bridge/releases/download/v${version}/${asset}`;
}

function explicitBinaryOverride() {
  const override = process.env.CLI_MEMORY_BINARY;
  if (!override) {
    return null;
  }

  return path.resolve(override);
}

function ensureParentDir(filePath) {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
}

function copyLocalBinary(sourcePath, destinationPath) {
  ensureParentDir(destinationPath);
  fs.copyFileSync(sourcePath, destinationPath);
  if (process.platform !== "win32") {
    fs.chmodSync(destinationPath, 0o755);
  }
}

function download(url, destPath) {
  return new Promise((resolve, reject) => {
    ensureParentDir(destPath);

    const request = https.get(
      url,
      {
        headers: {
          "user-agent": "cli-memory-npm-installer",
          accept: "application/octet-stream",
        },
      },
      (response) => {
        if (
          response.statusCode &&
          response.statusCode >= 300 &&
          response.statusCode < 400 &&
          response.headers.location
        ) {
          response.resume();
          download(response.headers.location, destPath).then(resolve, reject);
          return;
        }

        if (response.statusCode !== 200) {
          const chunks = [];
          response.on("data", (chunk) => chunks.push(chunk));
          response.on("end", () => {
            reject(
              new Error(
                `download failed with status ${response.statusCode}: ${Buffer.concat(chunks)
                  .toString("utf8")
                  .slice(0, 500)}`
              )
            );
          });
          return;
        }

        const file = fs.createWriteStream(destPath, { mode: 0o755 });
        response.pipe(file);

        file.on("finish", () => {
          file.close(() => resolve());
        });

        file.on("error", (error) => {
          fs.rmSync(destPath, { force: true });
          reject(error);
        });
      }
    );

    request.on("error", reject);
  });
}

async function main() {
  const version = packageVersion();
  const destination = vendorBinaryPath();
  const override = explicitBinaryOverride();

  if (fs.existsSync(destination)) {
    return;
  }

  if (override) {
    if (!fs.existsSync(override)) {
      console.error(`cli-memory: CLI_MEMORY_BINARY does not exist: ${override}`);
      process.exit(1);
    }

    copyLocalBinary(override, destination);
    return;
  }

  if (!supportedPlatform(process.platform, process.arch)) {
    console.error(
      `cli-memory: unsupported platform for npm binary install: ${platformKey(process.platform, process.arch)}`
    );
    process.exit(1);
  }

  const url = releaseUrl(version);
  process.stderr.write(`cli-memory: downloading ${url}\n`);

  try {
    await download(url, destination);
    if (process.platform !== "win32") {
      fs.chmodSync(destination, 0o755);
    }
  } catch (error) {
    console.error("cli-memory: failed to install bundled binary");
    console.error(error.message);
    process.exit(1);
  }
}

main();
