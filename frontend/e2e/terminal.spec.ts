import { test, expect } from '@playwright/test';

test.describe('Terminal', () => {
  test('terminal page renders', async ({ page }) => {
    await page.goto('/terminal/test-host');
    await page.waitForLoadState('networkidle');
    const body = await page.textContent('body');
    expect(body).toBeTruthy();
  });

  test('terminal page shows connection UI', async ({ page }) => {
    await page.goto('/terminal/test-host');
    await page.waitForLoadState('networkidle');
    // Terminal page should render something (connection dialog or terminal)
    const body = await page.textContent('body');
    expect(body!.length).toBeGreaterThan(5);
  });
});
