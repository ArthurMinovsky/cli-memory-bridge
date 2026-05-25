"use strict";

function binaryName(platform) {
  return platform === "win32" ? "cli-memory.exe" : "cli-memory";
}

function platformKey(platform, arch) {
  return `${platform}-${arch}`;
}

function releaseAssetName(version, platform, arch) {
  const name = binaryName(platform);
  return `${name}-${version}-${platformKey(platform, arch)}`;
}

function supportedPlatform(platform, arch) {
  return new Set([
    "darwin-arm64",
    "darwin-x64",
    "linux-arm64",
    "linux-x64",
    "win32-arm64",
    "win32-x64",
  ]).has(platformKey(platform, arch));
}

module.exports = {
  binaryName,
  platformKey,
  releaseAssetName,
  supportedPlatform,
};
