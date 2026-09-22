const { test, expect } = require('playwright/test');

test('loads the picking workspace and starts a retained solve', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByText('SolverForge Orders')).toBeVisible();
  await expect(page.getByRole('region', { name: 'Trolley route cards' })).toBeVisible();
  await expect(page.getByText('145 pick steps are unassigned')).toBeVisible();
  await page.getByRole('tab', { name: /Routes \/ Picking Plan/ }).click();
  await expect(page.getByText('Unassigned work').first()).toBeVisible();
  await page.getByRole('button', { name: 'Solve' }).click();
  await expect(page.locator('#sf-app')).toHaveAttribute('data-job-id', /.+/, { timeout: 15_000 });
  await expect(page.locator('#sf-app')).toHaveAttribute('data-lifecycle-state', 'SOLVING');
  await page.getByRole('button', { name: 'Pause' }).click();
  await expect(page.locator('#sf-app')).toHaveAttribute('data-lifecycle-state', 'PAUSED', { timeout: 15_000 });
  await page.getByRole('button', { name: 'Resume' }).click();
  await expect(page.locator('#sf-app')).toHaveAttribute('data-lifecycle-state', 'SOLVING');
  await page.getByRole('button', { name: 'Stop' }).click();
  await expect(page.locator('#sf-app')).toHaveAttribute('data-lifecycle-state', /CANCELLED|COMPLETED/, { timeout: 15_000 });
});
