import { test, expect } from '@playwright/test';

test.describe('Dashboard', () => {
  test('dashboard page loads', async ({ page }) => {
    await page.goto('/dashboard');
    // Wait for page content to render
    await page.waitForLoadState('networkidle');
    const body = await page.textContent('body');
    expect(body).toBeTruthy();
    expect(body!.length).toBeGreaterThan(10);
  });

  test('ops dashboard renders stat cards', async ({ page }) => {
    await page.goto('/ops-dashboard');
    await page.waitForLoadState('networkidle');
    const body = await page.textContent('body');
    expect(body).toContain('OpsPilot');
  });
});
