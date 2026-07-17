const assert = require("node:assert/strict");
const test = require("node:test");
const {
  classifyRemoteTag,
  isGitHubRemoteUrl,
  parseArgs,
  parseRemoteTagListing,
  releaseTagsForAllApps,
} = require("./publish-usecases.cjs");

test("publication mode requires exactly one tag selection", () => {
  assert.deepEqual(parseArgs(["--all"]), {
    all: true,
    branch: "main",
    dryRun: false,
  });
  assert.equal(parseArgs(["--tag", "solverforge-lessons@2.0.5"]).tag, "solverforge-lessons@2.0.5");
  assert.throws(() => parseArgs([]), /choose exactly one/);
  assert.throws(() => parseArgs(["--all", "--tag", "solverforge-lessons@2.0.5"]), /choose exactly one/);
  assert.throws(() => parseArgs(["--tag"]), /--tag requires a value/);
  assert.throws(() => parseArgs(["--all", "--remote"]), /--remote requires a value/);
});

test("only GitHub remotes qualify for Hugging Face workflow publication", () => {
  assert.equal(isGitHubRemoteUrl("https://github.com/SolverForge/solverforge-usecases"), true);
  assert.equal(isGitHubRemoteUrl("git@github.com:SolverForge/solverforge-usecases.git"), true);
  assert.equal(isGitHubRemoteUrl("http://localhost:3002/SolverForge/solverforge-usecases.git"), false);
});

test("annotated remote tags compare by their peeled commit", () => {
  const tagName = "solverforge-lessons@2.0.5";
  const listing = [
    `aaaaaaaa\trefs/tags/${tagName}`,
    `bbbbbbbb\trefs/tags/${tagName}^{}`,
  ].join("\n");
  const remoteTag = parseRemoteTagListing(listing, tagName);

  assert.deepEqual(remoteTag, { object: "aaaaaaaa", target: "bbbbbbbb" });
  assert.equal(classifyRemoteTag(remoteTag, "bbbbbbbb"), "existing");
  assert.equal(classifyRemoteTag(remoteTag, "cccccccc"), "conflict");
  assert.equal(classifyRemoteTag({}, "bbbbbbbb"), "new");
});

test("all-app publication derives one valid current release tag per allowlisted app", () => {
  const tags = releaseTagsForAllApps();
  assert.deepEqual(
    tags.map((tag) => tag.slice(0, tag.lastIndexOf("@"))),
    ["solverforge-deliveries", "solverforge-fsr", "solverforge-hospital", "solverforge-lessons"],
  );
  assert.equal(tags.every((tag) => /@\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?$/.test(tag)), true);
});
