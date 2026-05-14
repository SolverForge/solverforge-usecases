const APPS = {
  "uc-deliveries": {
    packageName: "solverforge-deliveries",
    spaceName: "solverforge-deliveries",
    displayName: "SolverForge Deliveries",
  },
  "uc-fsr": {
    packageName: "solverforge-fsr",
    spaceName: "solverforge-fsr",
    displayName: "SolverForge FSR",
  },
  "uc-hospital": {
    packageName: "solverforge-hospital",
    spaceName: "solverforge-hospital",
    displayName: "SolverForge Hospital",
  },
  "uc-lessons": {
    packageName: "solverforge-lessons",
    spaceName: "solverforge-lessons",
    displayName: "SolverForge Lessons",
  },
};

function appFolders() {
  return Object.keys(APPS);
}

function metadataForFolder(folder) {
  return APPS[folder] || null;
}

function metadataForPackage(packageName) {
  const folder = appFolders().find((candidate) => APPS[candidate].packageName === packageName);
  if (!folder) {
    return null;
  }
  return { folder, ...APPS[folder] };
}

function parseReleaseTag(tagName) {
  if (!tagName || !tagName.includes("@")) {
    return null;
  }

  const index = tagName.lastIndexOf("@");
  const packageName = tagName.slice(0, index);
  const version = tagName.slice(index + 1);
  const metadata = metadataForPackage(packageName);
  if (!metadata || !/^\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?$/.test(version)) {
    return null;
  }

  return { ...metadata, version, tagName };
}

module.exports = {
  APPS,
  appFolders,
  metadataForFolder,
  metadataForPackage,
  parseReleaseTag,
};
