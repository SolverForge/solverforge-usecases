#!/usr/bin/env node
// Per-app csemver release driver.
//
// Replaces scripts/release-usecase.cjs: the release toolchain moved from the
// commit-and-tag-version Node library to csemver, which reads a csemver.toml
// rendered per app rather than programmatic packageFiles/bumpFiles updaters.
// The rendered config lands as uc-<app>/csemver.toml (gitignored) and is
// removed after the run so apps never carry divergent copies.
//
// Behavior contract carried over from the old driver:
//   --app <uc-folder>            required
//   --release-as <level|semver>  forwarded to csemver -r
//   --dry-run                    preview, no writes
//   --prepared                   skip bump/changelog/commit (tag-only path);
//                                verifies Cargo.toml / Cargo.lock / CHANGELOG
//                                already agree before tagging
// The version is read from uc-<app>/Cargo.toml's `[package] version`.

import { execFileSync, spawnSync } from "node:child_process";
import { existsSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const CSEMVER = "csemver";

const apps = {
  "uc-deliveries": "solverforge-deliveries",
  "uc-fleet": "solverforge-fleet",
  "uc-flightcrew": "solverforge-flightcrew",
  "uc-fsr": "solverforge-fsr",
  "uc-furnace": "solverforge-furnace",
  "uc-hospital": "solverforge-hospital",
  "uc-lessons": "solverforge-lessons",
  "uc-orders": "solverforge-orders",
};

function usage() {
  console.error(
    `Usage: node scripts/release-usecase.mjs --app <uc-folder> [--release-as <version|major|minor|patch>] [--dry-run] [--prepared]\n\nOfficial apps: ${Object.keys(apps).join(", ")}\n`,
  );
}

function parseArgs(argv) {
  const args = { dryRun: false, prepared: false };
  for (let index = 2; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--app") args.app = argv[++index];
    else if (arg === "--release-as") args.releaseAs = argv[++index];
    else if (arg === "--dry-run") args.dryRun = true;
    else if (arg === "--prepared") args.prepared = true;
    else if (arg === "--help" || arg === "-h") args.help = true;
    else throw new Error(`unknown argument: ${arg}`);
  }
  return args;
}

function assertCleanWorktree() {
  const status = execFileSync("git", ["status", "--porcelain"], {
    encoding: "utf8",
  });
  if (status.trim()) {
    throw new Error(
      "release-usecase requires a clean worktree; commit or stash current changes before cutting a release",
    );
  }
}

// csemver has no --prepared mode: reproduce the old skip map (bump, changelog,
// and commit skipped, tag only) and verify the already-prepared surfaces by
// hand first, exactly as verify-usecase-release-tag.cjs does.
function verifyPrepared(app, packageName) {
  const cargo = path.join(app, "Cargo.toml");
  const lock = path.join(app, "Cargo.lock");
  const changelog = path.join(app, "CHANGELOG.md");

  const version = readFileSync(cargo, "utf8").match(
    /^\[package\][\s\S]*?^version\s*=\s*"([^"]+)"/m,
  )?.[1];
  if (!version) throw new Error(`${cargo} is missing [package] version`);

  const lockVersionPattern = new RegExp(
    `name\\s*=\\s*"${packageName}"\\s*\\n\\s*version\\s*=\\s*"([^"]+)"`,
  );
  const lockVersion = readFileSync(lock, "utf8").match(lockVersionPattern)?.[1];
  if (lockVersion !== version) {
    throw new Error(
      `${app}/Cargo.lock has ${packageName} at ${lockVersion ?? "no version"}, expected ${version}`,
    );
  }

  const heading = readFileSync(changelog, "utf8").match(/^## (\d+\.\d+\.\d+)/m);
  if (!heading || heading[1] !== version) {
    throw new Error(
      `${changelog} is missing a ## ${version} heading for the prepared version`,
    );
  }

  return version;
}

const TEMPLATE_PATH = "csemver.toml";

function renderAppConfig(template, app, packageName) {
  return template
    .replaceAll("@PKG@", `${packageName}@`)
    .replaceAll("@PACKAGE", packageName)
    .replaceAll("@APP", app);
}

function runCsemver(rendered, app, args, dryRun) {
  // Run from inside the app directory: csemver resolves bumpFiles paths
  // against the process cwd, and a bundle-root run silently drops entries
  // whose paths live under the app (verified: identical config bytes bump
  // both surfaces from uc-<app>/ but only the manifest from the root).
  const configPath = path.join(app, "csemver.toml");
  writeFileSync(configPath, rendered);
  const csemverArgs = ["-c", "csemver.toml", ...args];
  const result = spawnSync(CSEMVER, csemverArgs, {
    cwd: app,
    stdio: dryRun ? "pipe" : "inherit",
    encoding: "utf8",
  });
  rmSync(configPath, { force: true });
  if (result.status !== 0) {
    if (dryRun) {
      if (result.stdout) process.stdout.write(result.stdout);
      if (result.stderr) process.stderr.write(result.stderr);
    }
    process.exit(result.status ?? 1);
  }
  return dryRun ? result.stdout : "";
}

async function main() {
  const args = parseArgs(process.argv);
  if (args.help) return usage();
  if (!args.app) {
    usage();
    throw new Error("--app is required");
  }
  const packageName = apps[args.app];
  if (!packageName) {
    usage();
    throw new Error(`unknown app folder: ${args.app}`);
  }
  let csemverResolved = null;
  try {
    csemverResolved = execFileSync("which", [CSEMVER], { encoding: "utf8" }).trim();
  } catch {
    // not on PATH
  }
  if (!csemverResolved) {
    // csemver is resolved from PATH; give a pointed error when absent since it
    // is a native binary, not an npm dependency.
    throw new Error(
      "csemver not found on PATH; it is the C release manager this bundle releases with",
    );
  }

  if (!args.dryRun) assertCleanWorktree();
  if (args.prepared) {
    const version = verifyPrepared(args.app, packageName);
    console.error(`prepared release for ${packageName}@${version} verified`);
  }

  const template = readFileSync(TEMPLATE_PATH, "utf8");
  const rendered = renderAppConfig(template, args.app, packageName);

  const csemverArgs = [];
  if (args.prepared) {
    // Tag-only: current [package] version is the release version.
    csemverArgs.push("--skip.bump", "--skip.changelog");
  } else if (args.releaseAs) {
    csemverArgs.push("--release-as", args.releaseAs);
  }
  if (args.dryRun) csemverArgs.push("--dry-run");

  const stdout = runCsemver(rendered, args.app, csemverArgs, args.dryRun);
  if (args.dryRun) process.stdout.write(stdout);
}

main().catch((error) => {
  console.error(error.message);
  process.exit(1);
});
