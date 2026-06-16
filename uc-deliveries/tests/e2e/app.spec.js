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

function demoSelect(page) {
  return page.locator('.deliveries-field').filter({ hasText: 'Demo Data' }).locator('select');
}

test('boots the real deliveries app and serves required browser assets', async ({ page, request }) => {
  const errors = collectBrowserErrors(page);

  await expect(request.get('/health')).resolves.toBeOK();
  await expect(request.get('/sf/sf.js')).resolves.toBeOK();
  await expect(request.get('/sf/modules/sf-map.js')).resolves.toBeOK();
  await expect(request.get('/app.js')).resolves.toBeOK();
  await expect(request.get('/sf-config.json')).resolves.toBeOK();

  await page.goto('/');
  await expect(page).toHaveTitle('SolverForge Deliveries');
  await expect(page.getByText('Retained delivery-route optimization with route previews')).toBeVisible();
  await expect(page.locator('#sfStatusText')).toHaveText('Ready');
  await expect(page.locator('.sf-constraint-dot')).toHaveCount(4);

  for (const tab of ['Overview', 'By Vehicle', 'By Delivery', 'Data', 'REST API']) {
    await expect(page.getByRole('tab', { name: tab })).toBeVisible();
  }

  await expect(demoSelect(page)).toHaveValue('PHILADELPHIA');
  await expect(page.getByText('PHILADELPHIA · road network · 82 deliveries · 10 vehicles')).toBeVisible();
  await expect(page.locator('.deliveries-list__row')).toHaveCount(10);
  await expect(page.locator('#deliveries-map')).toBeVisible();
  await expect(page.locator('.sf-marker-vehicle')).toHaveCount(10);
  await expect(page.locator('.sf-marker-visit')).toHaveCount(82);

  expect(errors).toEqual([]);
});

test('switches datasets and exposes delivery-specific app panels', async ({ page }) => {
  const errors = collectBrowserErrors(page);

  await page.goto('/');
  await demoSelect(page).selectOption('HARTFORD');
  await expect(page.getByText('HARTFORD · road network · 50 deliveries · 10 vehicles')).toBeVisible();
  await expect(page.locator('.sf-marker-visit')).toHaveCount(50);

  await demoSelect(page).selectOption('FIRENZE');
  await expect(page.getByText('FIRENZE · road network · 80 deliveries · 10 vehicles')).toBeVisible();
  await expect(page.locator('.sf-marker-visit')).toHaveCount(80);

  await page.getByRole('tab', { name: 'Data' }).click();
  await expect(page.getByRole('heading', { name: 'Draft Data' })).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Vehicles' })).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Deliveries' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Recommend' }).first()).toBeVisible();
  await expect(page.locator('button').filter({ hasText: /Add|Remove/ })).toHaveCount(0);

  await page.getByRole('tab', { name: 'REST API' }).click();
  await expect(page.getByRole('heading', { name: 'GET /jobs/{id}/routes?snapshot_revision={n}' })).toBeVisible();
  await expect(page.getByRole('heading', { name: 'POST /recommendations/delivery-insertions' })).toBeVisible();

  expect(errors).toEqual([]);
});

test('highlights a route without moving or zooming the map', async ({ page }) => {
  const errors = collectBrowserErrors(page);

  await page.goto('/');
  await expect(page.locator('.sf-marker-visit')).toHaveCount(82);
  await page.evaluate(() => {
    window.__fitBoundsCalls = 0;
    const originalFitBounds = window.L.Map.prototype.fitBounds;
    window.L.Map.prototype.fitBounds = function (...args) {
      window.__fitBoundsCalls += 1;
      return originalFitBounds.apply(this, args);
    };
  });

  const firstRoute = page.locator('.deliveries-list__row').first();
  await firstRoute.click();

  await expect(firstRoute).toHaveClass(/is-focused/);
  await expect(firstRoute.getByRole('button')).toHaveText('Show All');
  await expect.poll(() => page.evaluate(() => window.__fitBoundsCalls)).toBe(0);

  expect(errors).toEqual([]);
});
