const { test, expect } = require('playwright/test');

// Live check: solving downloads (or reads the cached) OpenStreetMap road
// network, so it only runs through `make test-live-e2e`.
test.skip(!process.env.SOLVERFORGE_RUN_LIVE_TESTS, 'set SOLVERFORGE_RUN_LIVE_TESTS=1 to solve on the live road network');
test.setTimeout(300_000);

const transparentPng = Buffer.from(
  'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+/p9sAAAAASUVORK5CYII=',
  'base64',
);

test('solves the Bergamo plan with charging stops and road geometry', async ({ page }) => {
  const errors = [];
  page.on('pageerror', (error) => errors.push(error.message));
  page.on('console', (message) => {
    if (message.type() === 'error') errors.push(message.text());
  });
  await page.route('https://*.tile.openstreetmap.org/**', (route) =>
    route.fulfill({ status: 200, contentType: 'image/png', body: transparentPng }),
  );

  await page.goto('/');
  await expect(page.locator('.ev-route')).toHaveCount(6);

  await page.getByRole('button', { name: 'Solve' }).click();
  const app = page.locator('#sf-app');
  await expect(app).toHaveAttribute('data-job-id', /.+/, { timeout: 240_000 });
  await expect(app).toHaveAttribute('data-lifecycle-state', 'SOLVING');

  await page.getByRole('button', { name: 'Pause' }).click();
  await expect(app).toHaveAttribute('data-lifecycle-state', 'PAUSED', { timeout: 15_000 });
  await page.getByRole('button', { name: 'Resume' }).click();
  await expect(app).toHaveAttribute('data-lifecycle-state', 'SOLVING');
  await expect(app).toHaveAttribute('data-lifecycle-state', 'COMPLETED', { timeout: 60_000 });

  await expect(page.getByText('48/48 pharmacies routed', { exact: false })).toBeVisible();
  await expect(page.locator('.ev-route.has-violation')).toHaveCount(0);
  await expect(page.locator('.leaflet-overlay-pane path').first()).toBeVisible({ timeout: 30_000 });

  await page.getByRole('tab', { name: 'Schedule' }).click();
  await expect(page.getByText(/^Charge · /).first()).toBeVisible();

  expect(errors).toEqual([]);
});
