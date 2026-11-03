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
const port = Number(process.env.PLAYWRIGHT_PORT || 17961);
const baseURL = process.env.PLAYWRIGHT_BASE_URL || `http://127.0.0.1:${port}`;

module.exports = {
  testDir: '.',
  testMatch: '*.spec.js',
  workers: 1,
  timeout: 45_000,
  reporter: [['list']],
  use: {
    baseURL,
    browserName: 'chromium',
    screenshot: 'only-on-failure',
    trace: 'retain-on-failure',
    viewport: { width: 1440, height: 1000 },
    launchOptions: {
      executablePath: chromiumExecutable(),
      args: ['--no-sandbox'],
    },
  },
  webServer: {
    command: `PORT=${port} ${JSON.stringify(path.resolve(rootDir, process.env.CARGO_TARGET_DIR || 'target', 'release/solverforge-hospital'))}`,
    cwd: rootDir,
    url: `${baseURL}/health`,
    timeout: 20_000,
    reuseExistingServer: false,
  },
};
