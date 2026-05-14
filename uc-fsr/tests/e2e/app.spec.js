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

test('boots the real FSR app and serves required browser assets', async ({ page, request }) => {
  const errors = collectBrowserErrors(page);

  await expect(request.get('/health')).resolves.toBeOK();
  await expect(request.get('/info')).resolves.toBeOK();
  await expect(request.get('/sf/sf.js')).resolves.toBeOK();
  await expect(request.get('/sf/modules/sf-map.js')).resolves.toBeOK();
  await expect(request.get('/app.js')).resolves.toBeOK();
  await expect(request.get('/generated/ui-model.json')).resolves.toBeOK();
  await expect(request.get('/sf-config.json')).resolves.toBeOK();
  await expect((await request.get('/demo-data')).json()).resolves.toEqual({
    defaultId: 'STANDARD',
    availableIds: ['STANDARD'],
  });

  await page.goto('/');
  await expect(page).toHaveTitle('solverforge-fsr — SolverForge');
  await expect(page.getByText('SolverForge FSR')).toBeVisible();
  await expect(page.getByText('Bergamo technician routes, road travel, time windows, skills, and parts')).toBeVisible();
  await expect(page.locator('#sfStatusText')).toHaveText('Ready');
  await expect(page.locator('.sf-constraint-dot')).toHaveCount(10);

  for (const tab of ['Map', 'Routes', 'Data', 'REST API']) {
    await expect(page.getByRole('tab', { name: tab })).toBeVisible();
  }

  await expect(page.getByRole('cell', { name: 'STANDARD' })).toBeVisible();
  await expect(page.locator('.fsr-route-row')).toHaveCount(6);
  await expect(page.locator('.sf-marker-vehicle')).toHaveCount(6);
  await expect(page.locator('.sf-marker-visit')).toHaveCount(48);
  await expect(page.getByText('Unassigned visits')).toBeVisible();

  expect(errors).toEqual([]);
});

test('renders FSR-specific panels and visible REST API guide', async ({ page }) => {
  const errors = collectBrowserErrors(page);

  await page.goto('/');
  await page.getByRole('tab', { name: 'Routes' }).click();
  await expect(page.getByText('Bergamo Field Service Routes')).toBeVisible();
  await expect(page.locator('.sf-rail-timeline')).toBeVisible();

  await page.getByRole('tab', { name: 'Data' }).click();
  await expect(page.getByRole('heading', { name: 'Technician Routes' })).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Service Visits' })).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Locations' })).toBeVisible();

  await page.getByRole('tab', { name: 'REST API' }).click();
  await expect(page.getByRole('heading', { name: 'GET /demo-data/STANDARD' })).toBeVisible();
  await expect(page.getByRole('heading', { name: 'GET /jobs/{id}/routes?snapshot_revision={n}' })).toBeVisible();

  expect(errors).toEqual([]);
});

test('highlights a technician route without changing the route count', async ({ page }) => {
  const errors = collectBrowserErrors(page);

  await page.goto('/');
  const firstRoute = page.locator('.fsr-route-row').first();
  await firstRoute.click();

  await expect(firstRoute).toHaveClass(/is-focused/);
  await expect(firstRoute.getByRole('button')).toHaveText('Show All');
  await expect(page.locator('.fsr-route-row')).toHaveCount(6);

  expect(errors).toEqual([]);
});
