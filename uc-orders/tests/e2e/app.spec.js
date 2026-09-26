const { test, expect } = require('playwright/test');

test('loads the picking workspace and runs the retained solve lifecycle', async ({ page }) => {
  await page.goto('/');

  await expect(page.getByText('SolverForge Orders')).toBeVisible();
  await expect(page.locator('canvas.orders-warehouse__canvas')).toBeVisible();
  await expect(page.locator('.orders-legend__label').first()).toHaveText('Trolley 1');
  await expect(page.getByRole('heading', { name: 'Picking plan' })).toBeVisible();
  await expect(page.locator('.orders-card__empty').first()).toBeVisible();

  await page.getByRole('button', { name: 'Solve' }).click();
  await expect(page.locator('#sf-app')).toHaveAttribute('data-job-id', /.+/, { timeout: 15_000 });
  await expect(page.locator('#sf-app')).toHaveAttribute('data-lifecycle-state', 'SOLVING');

  // A running solve progressively fills the picking-plan cards.
  await expect(page.locator('.orders-card__item').first()).toBeVisible({ timeout: 20_000 });

  await page.getByRole('button', { name: 'Pause' }).click();
  await expect(page.locator('#sf-app')).toHaveAttribute('data-lifecycle-state', 'PAUSED', { timeout: 15_000 });
  await page.getByRole('button', { name: 'Resume' }).click();
  await expect(page.locator('#sf-app')).toHaveAttribute('data-lifecycle-state', 'SOLVING');
  await page.getByRole('button', { name: 'Stop' }).click();
  await expect(page.locator('#sf-app')).toHaveAttribute('data-lifecycle-state', /CANCELLED|COMPLETED/, { timeout: 15_000 });
});

test('generates a new dataset from the settings panel', async ({ page }) => {
  await page.goto('/');
  await expect(page.locator('canvas.orders-warehouse__canvas')).toBeVisible();

  await page.getByRole('button', { name: 'Data' }).click();
  await expect(page.locator('.orders-setting__range')).toHaveCount(3);
  await page.getByRole('button', { name: 'Generate New' }).click();

  await expect(page.locator('.orders-toolbar__summary')).toHaveText(/trolleys/, { timeout: 15_000 });
});
