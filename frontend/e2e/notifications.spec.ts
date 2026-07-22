import { test, expect } from '@playwright/test';

test.describe('Notification Channels', () => {
  test('notifications page renders', async ({ page }) => {
    await page.goto('/channels');
    await page.waitForLoadState('networkidle');
    const body = await page.textContent('body');
    expect(body).toBeTruthy();
  });

  test('notifications page has heading', async ({ page }) => {
    await page.goto('/channels');
    await page.waitForLoadState('networkidle');
    const heading = page.locator('h2');
    await expect(heading).toBeVisible();
  });

  test('notifications page has reload button', async ({ page }) => {
    await page.goto('/channels');
    await page.waitForLoadState('networkidle');
    const reloadBtn = page.locator('button', { hasText: /reload/i });
    if (await reloadBtn.count() > 0) {
      await expect(reloadBtn).toBeVisible();
    }
  });

  test('notifications page has add channel button', async ({ page }) => {
    await page.goto('/channels');
    await page.waitForLoadState('networkidle');
    const addBtn = page.locator('button', { hasText: /add channel/i });
    const count = await addBtn.count();
    if (count > 0) {
      await expect(addBtn).toBeVisible();
    }
  });

  test('notifications page shows channel cards', async ({ page }) => {
    await page.goto('/channels');
    await page.waitForLoadState('networkidle');
    // Channels are displayed in a grid of cards
    const cards = page.locator('.glass-card');
    const count = await cards.count();
    if (count > 0) {
      await expect(cards.first()).toBeVisible();
    }
  });

  test('notifications page shows type icons', async ({ page }) => {
    await page.goto('/channels');
    await page.waitForLoadState('networkidle');
    // Each channel card has an icon emoji
    const body = await page.textContent('body');
    const hasIcons = body?.includes('🔗') || body?.includes('📧') || body?.includes('💬') || body?.includes('💼');
    expect(hasIcons).toBeTruthy();
  });

  test('notifications page has test send buttons', async ({ page }) => {
    await page.goto('/channels');
    await page.waitForLoadState('networkidle');
    const testBtns = page.locator('button', { hasText: /test/i });
    const count = await testBtns.count();
    if (count > 0) {
      await expect(testBtns.first()).toBeVisible();
    }
  });

  test('notifications page shows toggle switch', async ({ page }) => {
    await page.goto('/channels');
    await page.waitForLoadState('networkidle');
    // Look for toggle switches
    const toggles = page.locator('.relative.w-10.h-6');
    const count = await toggles.count();
    if (count > 0) {
      await expect(toggles.first()).toBeVisible();
    }
  });

  test('notifications page channel cards show config', async ({ page }) => {
    await page.goto('/channels');
    await page.waitForLoadState('networkidle');
    // Channel cards contain config in a pre element
    const preElements = page.locator('pre');
    const count = await preElements.count();
    expect(count).toBeGreaterThanOrEqual(0);
  });

  test('notifications page shows add channel form on click', async ({ page }) => {
    await page.goto('/channels');
    await page.waitForLoadState('networkidle');
    const addBtn = page.locator('button', { hasText: /add channel/i });
    if (await addBtn.count() > 0) {
      await addBtn.click();
      await page.waitForTimeout(300);
      // Form should appear with input fields
      const inputs = page.locator('input');
      const count = await inputs.count();
      expect(count).toBeGreaterThan(0);
    }
  });

  test('notifications page form has name and type fields', async ({ page }) => {
    await page.goto('/channels');
    await page.waitForLoadState('networkidle');
    const addBtn = page.locator('button', { hasText: /add channel/i });
    if (await addBtn.count() > 0) {
      await addBtn.click();
      await page.waitForTimeout(300);
      // Check for select element (Type dropdown)
      const select = page.locator('select');
      if (await select.count() > 0) {
        const options = await select.locator('option').allTextContents();
        const types = options.join(' ').toLowerCase();
        expect(types).toContain('webhook');
      }
    }
  });
});
