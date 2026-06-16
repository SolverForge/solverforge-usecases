const path = require('node:path');

const rootDir = path.resolve(__dirname, '../..');
const port = Number(process.env.PLAYWRIGHT_PORT || 17960);
const baseURL = process.env.PLAYWRIGHT_BASE_URL || `http://127.0.0.1:${port}`;

module.exports = {
  testDir: '.',
  testMatch: '*.spec.js',
  testIgnore: process.env.SOLVERFORGE_RUN_LIVE_TESTS === '1' ? [] : ['*.live.spec.js'],
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
      executablePath: process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE || '/usr/bin/chromium',
      args: ['--no-sandbox'],
    },
  },
  webServer: {
    command: `PORT=${port} ${path.join(rootDir, 'target/release/solverforge_deliveries')}`,
    cwd: rootDir,
    url: `${baseURL}/health`,
    timeout: 20_000,
    reuseExistingServer: false,
  },
};
