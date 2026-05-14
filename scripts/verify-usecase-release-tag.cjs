#!/usr/bin/env node
const fs = require("fs");
const { parseReleaseTag } = require("./usecase-release-map.cjs");
const cargoUpdater = require("./cargo-package-version-updater.cjs");
const cargoLockUpdater = require("./cargo-lock-package-version-updater.cjs");

function writeGithubOutput(values) {
  const outputPath = process.env.GITHUB_OUTPUT;
  if (!outputPath) {
    return;
  }

  const lines = Object.entries(values).map(([key, value]) => `${key}=${value}`);
  fs.appendFileSync(outputPath, `${lines.join("\n")}\n`);
}

function main() {
  const tagName = process.argv[2] || process.env.GITHUB_REF_NAME || "";
  const parsed = parseReleaseTag(tagName);
  if (!parsed) {
    throw new Error(
      `release tag must be one of solverforge-deliveries@x.y.z, solverforge-fsr@x.y.z, solverforge-hospital@x.y.z, solverforge-lessons@x.y.z; got '${tagName}'`,
    );
  }

  const cargoPath = `${parsed.folder}/Cargo.toml`;
  const cargoLockPath = `${parsed.folder}/Cargo.lock`;
  const changelogPath = `${parsed.folder}/CHANGELOG.md`;
  const cargoVersion = cargoUpdater.readVersion(fs.readFileSync(cargoPath, "utf8"));
  if (cargoVersion !== parsed.version) {
    throw new Error(`${cargoPath} version is ${cargoVersion}, but tag ${tagName} declares ${parsed.version}`);
  }

  if (!fs.existsSync(cargoLockPath)) {
    throw new Error(`${cargoLockPath} is required for app-scoped releases`);
  }

  const cargoLockVersion = cargoLockUpdater.forPackage(parsed.packageName).readVersion(fs.readFileSync(cargoLockPath, "utf8"));
  if (cargoLockVersion !== parsed.version) {
    throw new Error(`${cargoLockPath} version for ${parsed.packageName} is ${cargoLockVersion}, but tag ${tagName} declares ${parsed.version}`);
  }

  if (!fs.existsSync(changelogPath)) {
    throw new Error(`${changelogPath} is required for app-scoped releases`);
  }

  const changelog = fs.readFileSync(changelogPath, "utf8");
  const headingPattern = new RegExp(`^## \\[?${parsed.version.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}\\]?\\b`, "m");
  if (!headingPattern.test(changelog)) {
    throw new Error(`${changelogPath} is missing a release heading for ${parsed.version}`);
  }

  const outputs = {
    folder: parsed.folder,
    package_name: parsed.packageName,
    space_name: parsed.spaceName,
    version: parsed.version,
    tag_name: parsed.tagName,
  };

  writeGithubOutput(outputs);
  console.log(`${parsed.tagName} -> ${parsed.folder} (${parsed.version})`);
}

try {
  main();
} catch (error) {
  console.error(error.message);
  process.exit(1);
}
