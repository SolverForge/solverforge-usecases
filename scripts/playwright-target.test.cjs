const assert = require('node:assert/strict');
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
