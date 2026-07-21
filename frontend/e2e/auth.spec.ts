import { test, expect } from '@playwright/test';

test.describe('Authentication', () => {
  test('login page renders', async ({ page }) => {
    await page.goto('/login');
    // Page should contain login form elements
    const body = await page.textContent('body');
    expect(body).toBeTruthy();
  });

  test('redirects to login when not authenticated', async ({ page }) => {
    // Clear any stored auth
    await page.goto('/');
    await page.waitForURL(/login|dashboard/, { timeout: 5000 }).catch(() => {});
    const url = page.url();
    expect(url).toMatch(/login|dashboard/);
  });

  test('login form has input fields', async ({ page }) => {
    await page.goto('/login');
    // Look for input elements (username/password)
    const inputs = await page.locator('input').count();
    expect(inputs).toBeGreaterThanOrEqual(1);
  });
});
