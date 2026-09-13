import AxeBuilder from '@axe-core/playwright';
import { expect, test, type Page } from '@playwright/test';

async function signIn(page: Page, email: string) {
  await page.goto('/app');
  await page.getByRole('button', { name: 'Log in' }).click();
  const dialog = page.getByRole('dialog', { name: 'Log in' });
  await dialog.getByLabel('Email').fill(email);
  await dialog.getByLabel('Password').fill('a-long-development-password');
  await dialog.getByRole('button', { name: 'Log in', exact: true }).click();
}

async function signInAsRoot(page: Page) {
  await signIn(page, 'root@example.test');
  await expect(page.getByRole('link', { name: 'Admin', exact: true })).toBeVisible();
}

test('root accounts can inspect and filter the user directory', async ({ page }) => {
  await signInAsRoot(page);
  await page.goto('/admin');

  await expect(page.getByRole('heading', { name: 'Users' })).toBeVisible();
  const table = page.getByRole('table');
  await expect(table.getByText('root@example.test')).toBeVisible();
  await expect(table.getByText('candidate@example.test')).toBeVisible();

  await page.getByLabel('Filter users').fill('candidate');
  await expect(table.getByText('candidate@example.test')).toBeVisible();
  await expect(table.getByText('root@example.test')).toHaveCount(0);
  await expect(page.locator('.list-count')).toContainText('1');
  await expect(page.locator('.list-count')).toContainText('matching users');

  const results = await new AxeBuilder({ page }).analyze();
  expect(results.violations).toEqual([]);
});

test('the user directory stays usable on a narrow viewport', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await signInAsRoot(page);
  await page.goto('/admin');
  await expect(page.getByRole('heading', { name: 'Users' })).toBeVisible();

  const widths = await page.evaluate(() => ({
    documentWidth: document.documentElement.scrollWidth,
    viewportWidth: window.innerWidth
  }));
  expect(widths.documentWidth).toBeLessThanOrEqual(widths.viewportWidth);
});

test('regular accounts see a clear access boundary', async ({ page }) => {
  await signIn(page, 'candidate@example.test');
  await page.goto('/admin');

  await expect(page.getByRole('heading', { name: 'Admin access is restricted' })).toBeVisible();
  await expect(page.getByRole('table')).toHaveCount(0);
});
