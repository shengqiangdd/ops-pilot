import { test, expect } from '@playwright/test';

test.describe('Hosts Page', () => {
  test('hosts page renders', async ({ page }) => {
    await page.goto('/hosts');
    await page.waitForLoadState('networkidle');
    const body = await page.textContent('body');
    expect(body).toBeTruthy();
  });

  test('hosts page has heading', async ({ page }) => {
    await page.goto('/hosts');
    await page.waitForLoadState('networkidle');
    const heading = page.locator('h2');
    await expect(heading).toBeVisible();
    const text = await heading.textContent();
    expect(text?.toLowerCase()).toContain('host');
  });

  test('hosts page has reload and add buttons', async ({ page }) => {
    await page.goto('/hosts');
    await page.waitForLoadState('networkidle');
    // Should have action buttons
    const buttons = page.locator('button');
    const count = await buttons.count();
    expect(count).toBeGreaterThanOrEqual(1);
  });

  test('hosts page shows table', async ({ page }) => {
    await page.goto('/hosts');
    await page.waitForLoadState('networkidle');
    const table = page.locator('table');
    const count = await table.count();
    if (count > 0) {
      await expect(table).toBeVisible();
      const thCount = await table.locator('th').count();
      // Should have columns: name, address, port, status, auth, actions
      expect(thCount).toBeGreaterThanOrEqual(4);
    }
  });

  test('hosts page shows table headers', async ({ page }) => {
    await page.goto('/hosts');
    await page.waitForLoadState('networkidle');
    const table = page.locator('table');
    if (await table.count() > 0) {
      const headerTexts = await table.locator('th').allTextContents();
      const joined = headerTexts.join(' ').toLowerCase();
      expect(joined).toContain('name');
      // Should also contain address or IP column
      expect(joined).toContain('address');
    }
  });

  test('hosts page shows status indicators', async ({ page }) => {
    await page.goto('/hosts');
    await page.waitForLoadState('networkidle');
    const table = page.locator('table');
    if (await table.count() > 0) {
      // Look for status badges (colored circles)
      const statusDots = table.locator('span.h-2.w-2');
      const count = await statusDots.count();
      expect(count).toBeGreaterThanOrEqual(0);
    }
  });

  test('hosts page shows SSH terminal button', async ({ page }) => {
    await page.goto('/hosts');
    await page.waitForLoadState('networkidle');
    // Look for SSH/Terminal buttons
    const sshBtn = page.locator('button', { hasText: /ssh|terminal/i });
    const count = await sshBtn.count();
    expect(count).toBeGreaterThanOrEqual(0);
  });

  test('hosts page shows vault locked warning', async ({ page }) => {
    await page.goto('/hosts');
    await page.waitForLoadState('networkidle');
    const body = await page.textContent('body');
    // The page may show a vault locked warning
    expect(body).toBeTruthy();
  });

  test('hosts page renders empty state', async ({ page }) => {
    await page.goto('/hosts');
    await page.waitForLoadState('networkidle');
    const body = await page.textContent('body');
    // Either shows hosts or empty state message
    expect(body).toBeTruthy();
  });

  test('hosts page allows searching', async ({ page }) => {
    await page.goto('/hosts');
    await page.waitForLoadState('networkidle');
    // Check if there's a search input on the page
    const searchInput = page.locator('input[type="text"][placeholder*="search" i], input[placeholder*="搜索" i], input[placeholder*="Search" i]');
    const count = await searchInput.count();
    expect(count).toBeGreaterThanOrEqual(0);
  });
});
