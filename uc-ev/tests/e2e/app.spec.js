const { test, expect } = require('playwright/test');

// Map tiles are decoration; serve a blank tile so the test never needs the
// internet. Nothing in this file solves, so no road network is downloaded.
const transparentPng = Buffer.from(
  'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+/p9sAAAAASUVORK5CYII=',
  'base64',
);

test.beforeEach(async ({ page }) => {
  await page.route('https://*.tile.openstreetmap.org/**', (route) =>
    route.fulfill({ status: 200, contentType: 'image/png', body: transparentPng }),
  );
});

function collectBrowserErrors(page) {
  const errors = [];
  page.on('pageerror', (error) => errors.push(error.message));
  page.on('console', (message) => {
    if (message.type() === 'error') errors.push(message.text());
  });
  return errors;
}

test('boots the EV workspace with the standard Bergamo plan', async ({ page, request }) => {
  const errors = collectBrowserErrors(page);
  for (const asset of ['/health', '/sf/sf.js', '/sf/modules/sf-map.js', '/app.js', '/app/main.mjs', '/sf-config.json']) {
    await expect(request.get(asset)).resolves.toBeOK();
  }

  await page.goto('/');
  await expect(page).toHaveTitle('SolverForge EV');
  await expect(page.getByText('Electric-van delivery routing with charging stops')).toBeVisible();
  await expect(page.locator('select')).toHaveCount(0);
  await expect(page.locator('.ev-route')).toHaveCount(6);
  await expect(page.getByText('0/48 pharmacies routed', { exact: false })).toBeVisible();
  // 48 pharmacies + 14 chargers as visit markers, one hub marker.
  await expect(page.locator('.sf-marker-visit')).toHaveCount(62);
  await expect(page.locator('.sf-marker-visit .fa-charging-station')).toHaveCount(14);

  expect(errors).toEqual([]);
});

test('every tab renders from the plan', async ({ page }) => {
  const errors = collectBrowserErrors(page);
  await page.goto('/');
  await expect(page.locator('.ev-route')).toHaveCount(6);

  await page.getByRole('tab', { name: 'Schedule' }).click();
  await expect(page.locator('.sf-rail-timeline')).toBeVisible();
  await expect(page.locator('.sf-rail-timeline').getByText('Van 1', { exact: true })).toBeVisible();

  await page.getByRole('tab', { name: 'Data' }).click();
  await expect(page.getByRole('heading', { name: 'Pharmacies' })).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Chargers' })).toBeVisible();

  await page.getByRole('tab', { name: 'REST API' }).click();
  await expect(page.getByText('GET /jobs/{id}/routes?snapshot_revision={n}')).toBeVisible();

  expect(errors).toEqual([]);
});
