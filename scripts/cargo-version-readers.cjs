// In-repo version readers shared by verify and publish tooling.
// Plain regex readers over the Cargo.toml / Cargo.lock surfaces; csemver is
// the writer, these are the read-side contract checks.
const SEMVER_PATTERN = /^\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?$/;

function readCargoPackageVersion(contents) {
  const match = contents.match(
    /(^\[package\][\s\S]*?^version\s*=\s*)(")([^"]+)(")/m,
  );
  if (!match) {
    throw new Error("Cargo.toml is missing [package] version");
  }
  return match[3];
}

function escapeRegExp(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function readCargoLockPackageVersion(contents, packageName) {
  const pattern = new RegExp(
    `name\\s*=\\s*"${escapeRegExp(packageName)}"\\s*\\n\\s*version\\s*=\\s*"([^"]+)"`,
  );
  const match = contents.match(pattern);
  if (!match) {
    throw new Error(
      `Cargo.lock is missing package '${packageName}' or its version`,
    );
  }
  return match[1];
}

module.exports = {
  SEMVER_PATTERN,
  readCargoPackageVersion,
  readCargoLockPackageVersion,
  escapeRegExp,
};
