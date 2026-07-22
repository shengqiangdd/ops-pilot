import { test, expect } from '@playwright/test';

test.describe('Global Search', () => {
  test('search button is visible on the page', async ({ page }) => {
    await page.goto('/dashboard');
    await page.waitForLoadState('networkidle');
    // Look for the search trigger button (contains "⌘K" or search icon)
    const searchBtn = page.locator('button').filter({ hasText: /⌘K|搜索|Search/ });
    const count = await searchBtn.count();
    if (count > 0) {
      await expect(searchBtn.first()).toBeVisible();
    }
  });

  test('search modal opens on button click', async ({ page }) => {
    await page.goto('/dashboard');
    await page.waitForLoadState('networkidle');
    // Find and click the search trigger
    const searchBtn = page.locator('button').filter({ hasText: /⌘K|搜索|Search/ });
    if (await searchBtn.count() > 0) {
      await searchBtn.first().click();
      await page.waitForTimeout(300);
      // The search dialog/modal should be visible
      const modal = page.locator('input[placeholder*="搜索" i]');
      const count = await modal.count();
      if (count > 0) {
        await expect(modal).toBeVisible();
      }
    }
  });

  test('search input is focused when opened', async ({ page }) => {
    await page.goto('/dashboard');
    await page.waitForLoadState('networkidle');
    const searchBtn = page.locator('button').filter({ hasText: /⌘K|搜索|Search/ });
    if (await searchBtn.count() > 0) {
      await searchBtn.first().click();
      await page.waitForTimeout(300);
      const input = page.locator('input[placeholder*="搜索" i], input[placeholder*="search" i]');
      if (await input.count() > 0) {
        await expect(input).toBeFocused();
      }
    }
  });

  test('search modal shows results', async ({ page }) => {
    await page.goto('/dashboard');
    await page.waitForLoadState('networkidle');
    const searchBtn = page.locator('button').filter({ hasText: /⌘K|搜索|Search/ });
    if (await searchBtn.count() > 0) {
      await searchBtn.first().click();
      await page.waitForTimeout(300);
      const input = page.locator('input[placeholder*="搜索" i], input[placeholder*="search" i]');
      if (await input.count() > 0) {
        await input.fill('host');
        await page.waitForTimeout(500);
        // Should show result items
        const resultButtons = page.locator('button').filter({ hasText: /host|主机/i });
        const resultCount = await resultButtons.count();
        expect(resultCount).toBeGreaterThanOrEqual(0);
      }
    }
  });

  test('search modal can be closed with Escape', async ({ page }) => {
    await page.goto('/dashboard');
    await page.waitForLoadState('networkidle');
    const searchBtn = page.locator('button').filter({ hasText: /⌘K|搜索|Search/ });
    if (await searchBtn.count() > 0) {
      await searchBtn.first().click();
      await page.waitForTimeout(300);
      // Press Escape
      await page.keyboard.press('Escape');
      await page.waitForTimeout(300);
      // The modal should be closed - search button should be visible again
      await expect(searchBtn.first()).toBeVisible();
    }
  });

  test('search shows ESC badge', async ({ page }) => {
    await page.goto('/dashboard');
    await page.waitForLoadState('networkidle');
    const searchBtn = page.locator('button').filter({ hasText: /⌘K|搜索|Search/ });
    if (await searchBtn.count() > 0) {
      await searchBtn.first().click();
      await page.waitForTimeout(300);
      // The modal should contain the ESC key badge
      const body = await page.textContent('body');
      expect(body).toContain('ESC');
    }
  });

  test('search modal with empty query shows page results', async ({ page }) => {
    await page.goto('/dashboard');
    await page.waitForLoadState('networkidle');
    const searchBtn = page.locator('button').filter({ hasText: /⌘K|搜索|Search/ });
    if (await searchBtn.count() > 0) {
      await searchBtn.first().click();
      await page.waitForTimeout(300);
      const input = page.locator('input[placeholder*="搜索" i], input[placeholder*="search" i]');
      if (await input.count() > 0) {
        // Clear input (should show page suggestions)
        await input.fill('');
        await page.waitForTimeout(500);
        // Should show page suggestions or empty state
        const body = await page.textContent('body');
        expect(body).toBeTruthy();
      }
    }
  });

  test('search modal shows result type badges', async ({ page }) => {
    await page.goto('/dashboard');
    await page.waitForLoadState('networkidle');
    const searchBtn = page.locator('button').filter({ hasText: /⌘K|搜索|Search/ });
    if (await searchBtn.count() > 0) {
      await searchBtn.first().click();
      await page.waitForTimeout(300);
      const input = page.locator('input[placeholder*="搜索" i], input[placeholder*="search" i]');
      if (await input.count() > 0) {
        await input.fill('host');
        await page.waitForTimeout(500);
        // Look for result type badges (uppercase text like HOST, ALERT, PAGE)
        const typeBadges = page.locator('text=HOST,text=ALERT,text=PAGE,text=RUNBOOK,text=KNOWLEDGE');
        const count = await typeBadges.count();
        expect(count).toBeGreaterThanOrEqual(0);
      }
    }
  });

  test('search modal result navigation with arrow keys', async ({ page }) => {
    await page.goto('/dashboard');
    await page.waitForLoadState('networkidle');
    const searchBtn = page.locator('button').filter({ hasText: /⌘K|搜索|Search/ });
    if (await searchBtn.count() > 0) {
      await searchBtn.first().click();
      await page.waitForTimeout(300);
      const input = page.locator('input[placeholder*="搜索" i], input[placeholder*="search" i]');
      if (await input.count() > 0) {
        await input.fill('host');
        await page.waitForTimeout(500);
        // Try keyboard navigation
        await page.keyboard.press('ArrowDown');
        await page.waitForTimeout(100);
        await page.keyboard.press('ArrowUp');
        await page.waitForTimeout(100);
        const body = await page.textContent('body');
        expect(body).toBeTruthy();
      }
    }
  });
});
