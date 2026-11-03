const fs = require('node:fs');
const path = require('node:path');

// Browser choice: PLAYWRIGHT_CHROMIUM_EXECUTABLE wins when set (an empty value
// selects Playwright's bundled Chromium, as in the Makefile), then the system
// Chromium; `undefined` makes Playwright launch its bundled Chromium (installed
// by `make install-e2e` when no system Chromium exists).
function chromiumExecutable() {
  const configured = process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE;
  if (configured !== undefined) return configured || undefined;
  return fs.existsSync('/usr/bin/chromium') ? '/usr/bin/chromium' : undefined;
}

const rootDir = path.resolve(__dirname, '../..');

module.exports = {
  testDir: '.',
  timeout: 30_000,
  workers: 1,
  use: {
    baseURL: 'http://127.0.0.1:7867',
    browserName: 'chromium',
    viewport: { width: 1440, height: 1000 },
    launchOptions: { executablePath: chromiumExecutable(), args: ['--no-sandbox'] },
  },
  webServer: {
    command: `PORT=7867 ${JSON.stringify(path.resolve(rootDir, process.env.CARGO_TARGET_DIR || 'target', 'release/solverforge-flightcrew'))}`,
    cwd: rootDir,
    url: 'http://127.0.0.1:7867/health',
    reuseExistingServer: false,
  },
};
