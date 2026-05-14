function packageVersionPattern() {
  return /(^\[package\][\s\S]*?^version\s*=\s*")([^"]+)(")/m;
}

function readVersion(contents) {
  const match = contents.match(packageVersionPattern());
  if (!match) {
    throw new Error("Cargo.toml is missing [package] version");
  }
  return match[2];
}

function writeVersion(contents, version) {
  if (!/^\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?$/.test(version)) {
    throw new Error(`invalid semver version: ${version}`);
  }

  const pattern = packageVersionPattern();
  if (!pattern.test(contents)) {
    throw new Error("Cargo.toml is missing [package] version");
  }
  return contents.replace(pattern, `$1${version}$3`);
}

module.exports = {
  readVersion,
  writeVersion,
};
