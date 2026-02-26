// tests/e2e/example.spec.ts
import { test, expect } from '@playwright/test';

test.describe('Basic Navigation', () => {
  test('should load the main page', async ({ page }) => {
    await page.goto('/');

    // 检查页面标题
    await expect(page).toHaveTitle(/wealthy/);

    // 检查是否存在主要导航元素
    await expect(page.locator('nav')).toBeVisible();
  });

  test('should navigate between main sections', async ({ page }) => {
    await page.goto('/');

    // 测试底部导航
    await page.locator('a[href="/records"]').click();
    await expect(page).toHaveURL(/\/records$/);

    await page.locator('a[href="/assets"]').click();
    await expect(page).toHaveURL(/\/assets$/);

    await page.locator('a[href="/stats"]').click();
    await expect(page).toHaveURL(/\/stats$/);
  });
});