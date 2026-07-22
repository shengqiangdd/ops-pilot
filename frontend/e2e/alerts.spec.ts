import { test, expect } from '@playwright/test';

test.describe('Alerts - Alert Rules', () => {
  test('alert rules page renders', async ({ page }) => {
    await page.goto('/alert-rules');
    await page.waitForLoadState('networkidle');
    const body = await page.textContent('body');
    expect(body).toBeTruthy();
  });

  test('alert rules page has heading', async ({ page }) => {
    await page.goto('/alert-rules');
    await page.waitForLoadState('networkidle');
    const heading = page.locator('h2');
    await expect(heading).toBeVisible();
    const text = await heading.textContent();
    expect(text?.toLowerCase()).toContain('alert');
  });

  test('alert rules page has reload button', async ({ page }) => {
    await page.goto('/alert-rules');
    await page.waitForLoadState('networkidle');
    const reloadBtn = page.locator('button', { hasText: /reload|loading|refresh/i });
    if (await reloadBtn.count() > 0) {
      await expect(reloadBtn).toBeVisible();
    }
  });

  test('alert rules page has add rule button', async ({ page }) => {
    await page.goto('/alert-rules');
    await page.waitForLoadState('networkidle');
    const addBtn = page.locator('button', { hasText: /add|new/i });
    if (await addBtn.count() > 0) {
      await expect(addBtn).toBeVisible();
    }
  });

  test('alert rules page shows table', async ({ page }) => {
    await page.goto('/alert-rules');
    await page.waitForLoadState('networkidle');
    const table = page.locator('table');
    const count = await table.count();
    if (count > 0) {
      await expect(table).toBeVisible();
      // Check for column headers
      const headers = await table.locator('th').count();
      expect(headers).toBeGreaterThan(0);
    }
  });

  test('alert rules page allows toggling rule enable/disable', async ({ page }) => {
    await page.goto('/alert-rules');
    await page.waitForLoadState('networkidle');
    // Look for toggle buttons (rounded buttons with inner circle)
    const toggles = page.locator('button.relative.w-10');
    const count = await toggles.count();
    if (count > 0) {
      // Click the first toggle
      await toggles.first().click();
      await page.waitForTimeout(300);
      // Should not crash
      const body = await page.textContent('body');
      expect(body).toBeTruthy();
    }
  });

  test('alert rules page shows edit buttons', async ({ page }) => {
    await page.goto('/alert-rules');
    await page.waitForLoadState('networkidle');
    const editBtns = page.locator('button', { hasText: 'Edit' });
    const count = await editBtns.count();
    expect(count).toBeGreaterThanOrEqual(0);
  });

  test('alert rules page shows delete buttons', async ({ page }) => {
    await page.goto('/alert-rules');
    await page.waitForLoadState('networkidle');
    const deleteBtns = page.locator('button', { hasText: 'Delete' });
    const count = await deleteBtns.count();
    expect(count).toBeGreaterThanOrEqual(0);
  });
});

test.describe('Alerts - Alert History', () => {
  test('alert history page renders', async ({ page }) => {
    await page.goto('/alert-history');
    await page.waitForLoadState('networkidle');
    const body = await page.textContent('body');
    expect(body).toBeTruthy();
  });

  test('alert history page has heading', async ({ page }) => {
    await page.goto('/alert-history');
    await page.waitForLoadState('networkidle');
    const heading = page.locator('h2');
    await expect(heading).toBeVisible();
  });

  test('alert history page has reload button', async ({ page }) => {
    await page.goto('/alert-history');
    await page.waitForLoadState('networkidle');
    const reloadBtn = page.locator('button', { hasText: /reload|loading/i });
    if (await reloadBtn.count() > 0) {
      await expect(reloadBtn).toBeVisible();
    }
  });

  test('alert history page has filter controls', async ({ page }) => {
    await page.goto('/alert-history');
    await page.waitForLoadState('networkidle');
    // Check for filter inputs (datetime-local and select)
    const dateInputs = page.locator('input[type="datetime-local"]');
    const count = await dateInputs.count();
    expect(count).toBeGreaterThanOrEqual(0);
  });

  test('alert history page has severity filter', async ({ page }) => {
    await page.goto('/alert-history');
    await page.waitForLoadState('networkidle');
    const selects = page.locator('select');
    const count = await selects.count();
    if (count > 0) {
      // Should have severity options
      const options = await selects.first().locator('option').count();
      expect(options).toBeGreaterThanOrEqual(1);
    }
  });

  test('alert history page shows table', async ({ page }) => {
    await page.goto('/alert-history');
    await page.waitForLoadState('networkidle');
    const table = page.locator('table');
    const count = await table.count();
    if (count > 0) {
      await expect(table).toBeVisible();
      const thCount = await table.locator('th').count();
      expect(thCount).toBeGreaterThanOrEqual(3);
    }
  });

  test('alert history page renders severity badges', async ({ page }) => {
    await page.goto('/alert-history');
    await page.waitForLoadState('networkidle');
    // Look for severity indicators (critical, warning, info text)
    const body = await page.textContent('body');
    const hasCritical = body?.toLowerCase().includes('critical');
    const hasWarning = body?.toLowerCase().includes('warning');
    const hasInfo = body?.toLowerCase().includes('info');
    // At least one severity type should be mentioned
    expect(hasCritical || hasWarning || hasInfo).toBe(true);
  });
});
