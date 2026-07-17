#!/usr/bin/env node
const { execFileSync, spawnSync } = require("child_process");
const fs = require("fs");
const path = require("path");
const cargoUpdater = require("./cargo-package-version-updater.cjs");
const { appFolders, metadataForFolder, parseReleaseTag } = require("./usecase-release-map.cjs");

function usage() {
  console.error(`Usage: node scripts/publish-usecases.cjs (--tag <release-tag> | --all) [--remote <git-remote>] [--branch <branch>] [--dry-run]

Pushes the release branch once and each new app tag in a separate Git command so
GitHub emits one Hugging Face sync event per use case.
`);
}

function parseArgs(argv) {
  const args = {
    all: false,
    branch: "main",
    dryRun: false,
  };

  function nextValue(option, index) {
    const value = argv[index + 1];
    if (!value || value.startsWith("--")) {
      throw new Error(`${option} requires a value`);
    }
    return value;
  }

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--tag") {
      args.tag = nextValue(arg, index);
      index += 1;
    } else if (arg === "--all") {
      args.all = true;
    } else if (arg === "--remote") {
      args.remote = nextValue(arg, index);
      index += 1;
    } else if (arg === "--branch") {
      args.branch = nextValue(arg, index);
      index += 1;
    } else if (arg === "--dry-run") {
      args.dryRun = true;
    } else if (arg === "--help" || arg === "-h") {
      args.help = true;
    } else {
      throw new Error(`unknown argument: ${arg}`);
    }
  }

  if (args.help) {
    return args;
  }
  if (Boolean(args.tag) === args.all) {
    throw new Error("choose exactly one of --tag <release-tag> or --all");
  }
  return args;
}

function gitOutput(args) {
  try {
    return execFileSync("git", args, {
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    }).trim();
  } catch (error) {
    const details = error.stderr?.trim() || error.stdout?.trim() || error.message;
    throw new Error(`git ${args.join(" ")} failed: ${details}`);
  }
}

function gitSucceeds(args) {
  return spawnSync("git", args, { stdio: "ignore" }).status === 0;
}

function runGit(args) {
  const result = spawnSync("git", args, { stdio: "inherit" });
  if (result.status !== 0) {
    throw new Error(`git ${args.join(" ")} failed with exit code ${result.status}`);
  }
}

function isGitHubRemoteUrl(url) {
  return /(?:\/\/|@)github\.com[/:]/i.test(url);
}

function resolveRemote(requestedRemote) {
  const remotes = gitOutput(["remote"])
    .split("\n")
    .map((remote) => remote.trim())
    .filter(Boolean)
    .map((remote) => ({ remote, url: gitOutput(["remote", "get-url", "--push", remote]) }));

  if (requestedRemote) {
    const selected = remotes.find(({ remote }) => remote === requestedRemote);
    if (!selected) {
      throw new Error(`Git remote '${requestedRemote}' does not exist`);
    }
    if (!isGitHubRemoteUrl(selected.url)) {
      throw new Error(
        `Git remote '${requestedRemote}' is not hosted on GitHub; the Hugging Face sync workflow requires a GitHub tag push`,
      );
    }
    return selected;
  }

  const githubRemotes = remotes.filter(({ url }) => isGitHubRemoteUrl(url));
  const canonical = githubRemotes.filter(({ url }) =>
    /[/:]SolverForge\/solverforge-usecases(?:\.git)?$/i.test(url),
  );
  const candidates = canonical.length > 0 ? canonical : githubRemotes;
  const selected =
    candidates.find(({ remote }) => remote === "solverforge") ||
    candidates.find(({ remote }) => remote === "origin") ||
    candidates[0];

  if (!selected) {
    throw new Error("no GitHub remote found; pass --remote <name> after configuring the canonical repository remote");
  }
  return selected;
}

function releaseTagsForAllApps() {
  return appFolders().map((folder) => {
    const metadata = metadataForFolder(folder);
    const cargoPath = `${folder}/Cargo.toml`;
    const version = cargoUpdater.readVersion(fs.readFileSync(cargoPath, "utf8"));
    return `${metadata.packageName}@${version}`;
  });
}

function verifyLocalTag(tagName, branch) {
  const parsed = parseReleaseTag(tagName);
  if (!parsed) {
    throw new Error(`invalid use-case release tag: ${tagName}`);
  }

  const verifier = path.join(__dirname, "verify-usecase-release-tag.cjs");
  execFileSync(process.execPath, [verifier, tagName], { stdio: "inherit" });

  const tagRef = `refs/tags/${tagName}`;
  const type = gitOutput(["cat-file", "-t", tagRef]);
  if (type !== "tag") {
    throw new Error(`${tagName} must be an annotated tag; found Git object type '${type}'`);
  }

  const target = gitOutput(["rev-parse", `${tagRef}^{}`]);
  if (!gitSucceeds(["merge-base", "--is-ancestor", target, `refs/heads/${branch}`])) {
    throw new Error(`${tagName} points outside refs/heads/${branch}`);
  }
  if (!gitSucceeds(["cat-file", "-e", `${target}:.github/workflows/sync-hf-spaces.yml`])) {
    throw new Error(`${tagName} does not contain .github/workflows/sync-hf-spaces.yml`);
  }

  return { ...parsed, ref: tagRef, target };
}

function parseRemoteTagListing(output, tagName) {
  const tagRef = `refs/tags/${tagName}`;
  const peeledRef = `${tagRef}^{}`;
  const result = {};

  for (const line of output.split("\n").filter(Boolean)) {
    const [object, ref] = line.split(/\s+/);
    if (ref === tagRef) {
      result.object = object;
    } else if (ref === peeledRef) {
      result.target = object;
    }
  }

  if (result.object && !result.target) {
    result.target = result.object;
  }
  return result;
}

function classifyRemoteTag(remoteTag, localTarget) {
  if (!remoteTag.object) {
    return "new";
  }
  return remoteTag.target === localTarget ? "existing" : "conflict";
}

function inspectRemoteTag(remote, localTag) {
  const listing = gitOutput([
    "ls-remote",
    "--tags",
    remote,
    localTag.ref,
    `${localTag.ref}^{}`,
  ]);
  const remoteTag = parseRemoteTagListing(listing, localTag.tagName);
  const state = classifyRemoteTag(remoteTag, localTag.target);
  if (state === "conflict") {
    throw new Error(
      `${localTag.tagName} already exists on ${remote} at ${remoteTag.target}, not ${localTag.target}; refusing to overwrite it`,
    );
  }
  return { ...localTag, remoteState: state };
}

function inspectBranch(remote, branch) {
  const localRef = `refs/heads/${branch}`;
  if (!gitSucceeds(["show-ref", "--verify", "--quiet", localRef])) {
    throw new Error(`local branch ${localRef} does not exist`);
  }

  const currentBranch = gitOutput(["branch", "--show-current"]);
  if (currentBranch !== branch) {
    throw new Error(`publication must run from branch '${branch}', but the current branch is '${currentBranch || "detached"}'`);
  }

  const localTarget = gitOutput(["rev-parse", localRef]);
  const remoteLine = gitOutput(["ls-remote", "--heads", remote, localRef]);
  const remoteTarget = remoteLine ? remoteLine.split(/\s+/)[0] : null;

  if (remoteTarget && remoteTarget !== localTarget) {
    if (!gitSucceeds(["cat-file", "-e", `${remoteTarget}^{commit}`])) {
      gitOutput(["fetch", "--quiet", "--no-tags", remote, localRef]);
    }
    if (!gitSucceeds(["merge-base", "--is-ancestor", remoteTarget, localTarget])) {
      throw new Error(`${remote}/${branch} is not an ancestor of local ${branch}; refusing a non-fast-forward publication`);
    }
  }

  return {
    localRef,
    localTarget,
    remoteTarget,
    needsPush: remoteTarget !== localTarget,
  };
}

function assertCleanWorktree() {
  if (gitOutput(["status", "--porcelain"])) {
    throw new Error("publication requires a clean worktree");
  }
}

function printPlan(remote, branch, tags, dryRun) {
  console.log("\nUse-case publication plan");
  console.log(`  GitHub remote: ${remote.remote} (${remote.url})`);
  console.log(
    `  Branch: ${branch.localRef} ${branch.needsPush ? `${branch.remoteTarget || "(missing)"} -> ${branch.localTarget}` : "already current"}`,
  );
  for (const tag of tags) {
    console.log(`  Tag: ${tag.tagName} -> ${tag.target} (${tag.remoteState})`);
  }
  if (dryRun) {
    console.log("\nDry run complete; no refs were pushed.");
  }
}

function publish(remote, branch, tags) {
  if (branch.needsPush) {
    console.log(`\nPushing ${branch.localRef} to ${remote.remote}...`);
    runGit(["push", remote.remote, `${branch.localRef}:${branch.localRef}`]);
  }

  for (const tag of tags) {
    if (tag.remoteState === "existing") {
      console.log(`Skipping ${tag.tagName}; the remote tag already points to ${tag.target}.`);
      continue;
    }
    console.log(`Pushing ${tag.tagName} separately so GitHub emits its release event...`);
    runGit(["push", remote.remote, `${tag.ref}:${tag.ref}`]);
  }

  console.log("\nPublication refs pushed. GitHub Actions now owns the Hugging Face Space sync.");
}

function main() {
  const args = parseArgs(process.argv.slice(2));
  if (args.help) {
    usage();
    return;
  }

  assertCleanWorktree();
  const remote = resolveRemote(args.remote);
  const tagNames = args.all ? releaseTagsForAllApps() : [args.tag];
  const localTags = tagNames.map((tagName) => verifyLocalTag(tagName, args.branch));
  const branch = inspectBranch(remote.remote, args.branch);
  const tags = localTags.map((tag) => inspectRemoteTag(remote.remote, tag));

  printPlan(remote, branch, tags, args.dryRun);
  if (!args.dryRun) {
    publish(remote, branch, tags);
  }
}

if (require.main === module) {
  try {
    main();
  } catch (error) {
    console.error(error.message);
    process.exit(1);
  }
}

module.exports = {
  classifyRemoteTag,
  isGitHubRemoteUrl,
  parseArgs,
  parseRemoteTagListing,
  releaseTagsForAllApps,
};
