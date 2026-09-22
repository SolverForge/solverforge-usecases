const path = require('node:path');
const rootDir = path.resolve(__dirname, '../..');

module.exports = {
  testDir: '.',
  timeout: 30_000,
  workers: 1,
  use: {
    baseURL: 'http://127.0.0.1:7868',
    browserName: 'chromium',
    viewport: { width: 1440, height: 1000 },
    launchOptions: { executablePath: process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE || '/usr/bin/chromium', args: ['--no-sandbox'] },
  },
  webServer: {
    command: `PORT=7868 ${path.join(rootDir, 'target/release/solverforge-orders')}`,
    cwd: rootDir,
    url: 'http://127.0.0.1:7868/health',
    reuseExistingServer: false,
  },
};
