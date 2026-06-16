const { test, expect } = require('playwright/test');

const transparentPng = Buffer.from(
  'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+/p9sAAAAASUVORK5CYII=',
  'base64',
);

test.beforeEach(async ({ page }) => {
  await page.route('https://*.tile.openstreetmap.org/**', (route) => route.fulfill({
    status: 200,
    contentType: 'image/png',
    body: transparentPng,
  }));
});

function collectBrowserErrors(page) {
  const errors = [];
  page.on('pageerror', (error) => errors.push(error.message));
  page.on('console', (message) => {
    if (message.type() === 'error') errors.push(message.text());
  });
  return errors;
}

test('starts a retained road-network solve and returns control to the user', async ({ page }) => {
  const errors = collectBrowserErrors(page);

  await page.goto('/');
  await page.locator('button').filter({ hasText: 'Solve' }).first().click();

  await expect(page.locator('#sf-app')).toHaveAttribute('data-job-id', /.+/, { timeout: 10_000 });
  const stopButton = page.locator('button').filter({ hasText: 'Stop' }).first();
  if (await stopButton.isVisible()) {
    await stopButton.click();
  }
  await expect(page.locator('button').filter({ hasText: 'Solve' }).first()).toBeVisible({ timeout: 15_000 });

  expect(errors).toEqual([]);
});
