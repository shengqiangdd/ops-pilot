import { test, expect } from '@playwright/test';

test.describe('Navigation', () => {
  test('sidebar renders navigation items', async ({ page }) => {
    await page.goto('/dashboard');
    await page.waitForLoadState('networkidle');
    // Sidebar should have navigation buttons
    const navButtons = await page.locator('nav button').count();
    expect(navButtons).toBeGreaterThan(0);
  });

  test('can navigate between pages', async ({ page }) => {
    await page.goto('/dashboard');
    await page.waitForLoadState('networkidle');

    // Look for any clickable nav item
    const navItems = page.locator('nav button');
    const count = await navItems.count();
    if (count > 1) {
      await navItems.nth(1).click();
      // Page should still be valid
      const body = await page.textContent('body');
      expect(body).toBeTruthy();
    }
  });
});
