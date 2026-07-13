#!/usr/bin/env node
const commitAndTagVersion = require("commit-and-tag-version");
const { execFileSync } = require("child_process");
const fs = require("fs");
const path = require("path");
const cargoUpdater = require("./cargo-package-version-updater.cjs");
const cargoLockUpdater = require("./cargo-lock-package-version-updater.cjs");
const { metadataForFolder, appFolders } = require("./usecase-release-map.cjs");

function usage() {
  const apps = appFolders().join(", ");
  console.error(`Usage: node scripts/release-usecase.cjs --app <uc-folder> [--release-as <version|major|minor|patch>] [--dry-run] [--first-release] [--prepared]

Official apps: ${apps}
`);
}

function parseArgs(argv) {
  const args = {
    dryRun: false,
    firstRelease: false,
    prepared: false,
  };

  for (let index = 2; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--app") {
      args.app = argv[++index];
    } else if (arg === "--release-as") {
      args.releaseAs = argv[++index];
    } else if (arg === "--dry-run") {
      args.dryRun = true;
    } else if (arg === "--first-release") {
      args.firstRelease = true;
    } else if (arg === "--prepared") {
      args.prepared = true;
    } else if (arg === "--help" || arg === "-h") {
      args.help = true;
    } else {
      throw new Error(`unknown argument: ${arg}`);
    }
  }

  return args;
}

function cargoFile(app) {
  return {
    filename: `${app}/Cargo.toml`,
    updater: cargoUpdater,
  };
}

function cargoLockFile(app, packageName) {
  return {
    filename: `${app}/Cargo.lock`,
    updater: cargoLockUpdater.forPackage(packageName),
  };
}

function assertCleanWorktree() {
  const status = execFileSync("git", ["status", "--porcelain"], { encoding: "utf8" });
  if (status.trim()) {
    throw new Error("release-usecase requires a clean worktree; commit or stash current changes before cutting a release");
  }
}

function verifyPreparedRelease(cargo, metadata) {
  const contents = fs.readFileSync(cargo.filename, "utf8");
  const version = cargo.updater.readVersion(contents);
  const tagName = `${metadata.packageName}@${version}`;
  const verifier = path.join(__dirname, "verify-usecase-release-tag.cjs");

  execFileSync(process.execPath, [verifier, tagName], { stdio: "inherit" });
}

async function main() {
  const args = parseArgs(process.argv);
  if (args.help) {
    usage();
    return;
  }

  const metadata = metadataForFolder(args.app);
  if (!metadata) {
    usage();
    throw new Error(`unknown app folder: ${args.app || "(missing)"}`);
  }

  if (args.prepared && (args.firstRelease || args.releaseAs)) {
    throw new Error("--prepared cannot be combined with --first-release or --release-as");
  }

  if (!args.dryRun) {
    assertCleanWorktree();
  }

  const cargo = cargoFile(args.app);
  const cargoLock = cargoLockFile(args.app, metadata.packageName);

  if (args.prepared) {
    verifyPreparedRelease(cargo, metadata);
  }

  const options = {
    path: args.app,
    infile: `${args.app}/CHANGELOG.md`,
    tagPrefix: `${metadata.packageName}@`,
    packageFiles: [cargo],
    bumpFiles: [cargo, cargoLock],
    releaseCommitMessageFormat: `chore(${metadata.packageName}): release {{currentTag}}`,
    header: "# Changelog\n\nAll notable changes to this use case are documented in this file.\n",
    dryRun: args.dryRun,
    firstRelease: args.firstRelease || args.prepared,
  };

  if (args.prepared) {
    options.skip = { changelog: true, commit: true };
  }

  if (args.releaseAs) {
    options.releaseAs = args.releaseAs;
  }

  await commitAndTagVersion(options);
}

main().catch((error) => {
  console.error(error.message);
  process.exit(1);
});
