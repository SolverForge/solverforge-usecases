const { test, expect } = require('playwright/test');

function collectBrowserErrors(page) {
  const errors = [];
  page.on('pageerror', (error) => errors.push(error.message));
  page.on('console', (message) => {
    if (message.type() === 'error') errors.push(message.text());
  });
  return errors;
}

test('boots the real hospital app and serves required browser assets', async ({ page, request }) => {
  const errors = collectBrowserErrors(page);

  await expect(request.get('/health')).resolves.toBeOK();
  await expect(request.get('/info')).resolves.toBeOK();
  await expect(request.get('/sf/sf.js')).resolves.toBeOK();
  await expect(request.get('/sf/sf.css')).resolves.toBeOK();
  await expect(request.get('/app/main.mjs')).resolves.toBeOK();
  await expect(request.get('/generated/ui-model.json')).resolves.toBeOK();
  await expect(request.get('/sf-config.json')).resolves.toBeOK();
  await expect((await request.get('/demo-data')).json()).resolves.toEqual(['LARGE']);

  await page.goto('/');
  await expect(page).toHaveTitle('SolverForge Hospital — SolverForge');
  await expect(page.getByText('SolverForge Hospital')).toBeVisible();
  await expect(page.getByText('Constraint Optimizer')).toBeVisible();
  await expect(page.locator('#sfStatusText')).toHaveText('Ready');
  await expect(page.locator('.sf-constraint-dot')).toHaveCount(9);

  for (const tab of ['By location', 'By employee', 'Data', 'REST API']) {
    await expect(page.getByRole('tab', { name: tab })).toBeVisible();
  }

  await expect(page.getByText('Location schedule')).toBeVisible();
  await expect(page.locator('.sf-rail-timeline')).toHaveCount(2);
  await expect(page.locator('.sf-rail-timeline-row').first()).toBeVisible();
  await expect(page.locator('.sf-rail-timeline-item').first()).toBeVisible();

  expect(errors).toEqual([]);
});

test('renders hospital-specific views and the visible REST API guide', async ({ page }) => {
  const errors = collectBrowserErrors(page);

  await page.goto('/');
  await page.getByRole('tab', { name: 'By employee' }).click();
  await expect(page.getByText('Employee schedule')).toBeVisible();
  await expect(page.locator('#view-by-employee .sf-rail-timeline-item').first()).toBeVisible();

  await page.getByRole('tab', { name: 'Data' }).click();
  await expect(page.getByRole('heading', { name: 'Shifts' })).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Employees' })).toBeVisible();
  await expect(page.getByText('CAREHUB')).toBeVisible();
  await expect(page.getByText('EMPLOYEEIDX')).toBeVisible();

  await page.getByRole('tab', { name: 'REST API' }).click();
  await expect(page.getByRole('heading', { name: 'GET /demo-data/LARGE' })).toBeVisible();
  await expect(page.getByRole('heading', { name: 'GET /jobs/{id}/events' })).toBeVisible();
  await expect(page.getByRole('heading', { name: 'GET /jobs/{id}/analysis?snapshot_revision={n}' })).toBeVisible();

  expect(errors).toEqual([]);
});

test('starts a retained solve and returns control to the user', async ({ page }) => {
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
