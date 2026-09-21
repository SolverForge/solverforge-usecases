const { test, expect } = require('playwright/test');

test('loads the crew workspace and starts a retained solve', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByText('SolverForge Flight Crew')).toBeVisible();
  await expect(page.getByRole('region', { name: 'Crew rotations' })).toBeVisible();
  await expect(page.getByText('Unassigned seats').first()).toBeVisible();
  await page.getByRole('button', { name: 'Solve' }).click();
  await expect(page.locator('#sf-app')).toHaveAttribute('data-job-id', /.+/, { timeout: 15_000 });
  await expect(page.locator('#sf-app')).toHaveAttribute('data-lifecycle-state', /SOLVING|COMPLETED/);
});
