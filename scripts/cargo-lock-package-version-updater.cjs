const SEMVER_PATTERN = /^\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?$/;

function escapeRegExp(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function packageNamePattern(packageName) {
  return new RegExp(`^name\\s*=\\s*"${escapeRegExp(packageName)}"\\s*$`, "m");
}

function packageBlocks(contents) {
  return contents.split(/(?=^\[\[package\]\]\s*$)/m);
}

function packageBlockIndexes(contents, packageName) {
  const blocks = packageBlocks(contents);
  const indexes = [];

  for (let index = 0; index < blocks.length; index += 1) {
    if (packageNamePattern(packageName).test(blocks[index])) {
      indexes.push(index);
    }
  }

  return { blocks, indexes };
}

function singlePackageBlock(contents, packageName) {
  const { blocks, indexes } = packageBlockIndexes(contents, packageName);
  if (indexes.length === 0) {
    throw new Error(`Cargo.lock is missing package '${packageName}'`);
  }
  if (indexes.length > 1) {
    throw new Error(`Cargo.lock has multiple package entries named '${packageName}'`);
  }

  return { blocks, index: indexes[0] };
}

function forPackage(packageName) {
  return {
    readVersion(contents) {
      const { blocks, index } = singlePackageBlock(contents, packageName);
      const match = blocks[index].match(/^version\s*=\s*"([^"]+)"\s*$/m);
      if (!match) {
        throw new Error(`Cargo.lock package '${packageName}' is missing a version`);
      }

      return match[1];
    },

    writeVersion(contents, version) {
      if (!SEMVER_PATTERN.test(version)) {
        throw new Error(`invalid semver version: ${version}`);
      }

      const { blocks, index } = singlePackageBlock(contents, packageName);
      if (!/^version\s*=\s*"[^"]+"\s*$/m.test(blocks[index])) {
        throw new Error(`Cargo.lock package '${packageName}' is missing a version`);
      }

      blocks[index] = blocks[index].replace(/^version\s*=\s*"[^"]+"\s*$/m, `version = "${version}"`);
      return blocks.join("");
    },
  };
}

module.exports = {
  forPackage,
};
