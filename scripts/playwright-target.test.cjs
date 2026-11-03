const assert = require('node:assert/strict');
const fs = require('node:fs');
const { test } = require('node:test');
const path = require('node:path');

for (const app of ['deliveries', 'flightcrew', 'fsr', 'hospital', 'orders']) {
  test(`${app} browser server follows Cargo's target directory`, () => {
    const configPath = path.resolve(__dirname, `../uc-${app}/tests/e2e/playwright.config.js`);
    const root = path.resolve(__dirname, `../uc-${app}`);
    const previous = process.env.CARGO_TARGET_DIR;
    try {
      for (const target of [undefined, '/shared/cargo target', 'custom-target']) {
        if (target === undefined) delete process.env.CARGO_TARGET_DIR;
        else process.env.CARGO_TARGET_DIR = target;
        delete require.cache[configPath];
        const config = require(configPath);
        const expected = path.resolve(root, target || 'target', 'release');
        assert.ok(config.webServer.command.includes(expected), config.webServer.command);
      }
    } finally {
      if (previous === undefined) delete process.env.CARGO_TARGET_DIR;
      else process.env.CARGO_TARGET_DIR = previous;
      delete require.cache[configPath];
    }
  });
}

for (const app of ['deliveries', 'flightcrew', 'fsr', 'hospital', 'orders']) {
  test(`${app} browser tests honor PLAYWRIGHT_CHROMIUM_EXECUTABLE`, () => {
    const configPath = path.resolve(__dirname, `../uc-${app}/tests/e2e/playwright.config.js`);
    const previous = process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE;
    const load = () => {
      delete require.cache[configPath];
      return require(configPath).use.launchOptions.executablePath;
    };
    try {
      process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE = '/opt/custom/chromium';
      assert.equal(load(), '/opt/custom/chromium');
      // An empty value selects Playwright's bundled Chromium.
      process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE = '';
      assert.equal(load(), undefined);
      delete process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE;
      const systemChromium = fs.existsSync('/usr/bin/chromium') ? '/usr/bin/chromium' : undefined;
      assert.equal(load(), systemChromium);
    } finally {
      if (previous === undefined) delete process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE;
      else process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE = previous;
      delete require.cache[configPath];
    }
  });
}
