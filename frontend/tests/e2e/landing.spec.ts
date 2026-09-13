import AxeBuilder from '@axe-core/playwright';
import { expect, test } from '@playwright/test';

test('shows the CV Workbench landing page and routes visitors to the builder', async ({ page }) => {
  const apiRequests: string[] = [];
  page.on('request', (request) => {
    if (request.url().includes('/api/')) apiRequests.push(request.url());
  });

  await page.goto('/');
  await expect(
    page.getByRole('heading', { name: /Build the CV\. See the result\./ })
  ).toBeVisible();
  await expect(
    page.getByText(
      'Enter your experience, review the generated LaTeX, and download the finished PDF.'
    )
  ).toBeVisible();
  await expect(page.getByRole('region', { name: 'CV form builder' })).toHaveCount(0);
  await expect(page.getByRole('link', { name: 'Open builder' })).toHaveAttribute('href', '/app');
  await expect(page.locator('link[rel="canonical"]')).toHaveAttribute('href', /\/$/);
  await expect(page.locator('meta[property="og:type"]')).toHaveAttribute('content', 'website');
  await expect(page.locator('meta[name="twitter:card"]')).toHaveAttribute(
    'content',
    'summary_large_image'
  );
  expect(
    await page.locator('script[type="application/ld+json"]').evaluate((script) => script.innerHTML)
  ).toContain('CV Workbench');
  expect(apiRequests).toEqual([]);

  const previews = page.getByRole('img', { name: /CV template preview/ });
  await expect(previews).toHaveCount(3);
  for (let index = 0; index < 3; index += 1) {
    expect(
      await previews.nth(index).evaluate((image) => (image as HTMLImageElement).naturalWidth)
    ).toBeGreaterThan(0);
  }
});

test('opens the builder from the primary landing CTA', async ({ page }) => {
  await page.goto('/');
  await Promise.all([
    page.waitForURL('**/app'),
    page.getByRole('link', { name: 'Start building' }).first().click()
  ]);
  await expect(page.getByRole('region', { name: 'CV form builder' })).toBeVisible();
});

test('keeps the interactive builder out of search results', async ({ page }) => {
  await page.goto('/app');
  await expect(page.locator('meta[name="robots"]')).toHaveAttribute(
    'content',
    'noindex, nofollow, noarchive'
  );
});

test('keeps the landing page usable without horizontal overflow on mobile', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/');
  await expect(
    page.getByRole('heading', { name: /Build the CV\. See the result\./ })
  ).toBeVisible();
  const previews = page.getByRole('img', { name: /CV template preview/ });
  await expect(previews).toHaveCount(3);
  for (let index = 0; index < 3; index += 1) {
    expect(
      await previews.nth(index).evaluate((image) => (image as HTMLImageElement).naturalWidth)
    ).toBeGreaterThan(0);
  }
  const widths = await page.evaluate(() => ({
    documentWidth: document.documentElement.scrollWidth,
    viewportWidth: window.innerWidth
  }));
  expect(widths.documentWidth).toBeLessThanOrEqual(widths.viewportWidth);
});

test('has no automatically detectable landing-page accessibility violations', async ({ page }) => {
  await page.goto('/');
  await expect(
    page.getByRole('heading', { name: /Build the CV\. See the result\./ })
  ).toBeVisible();
  const results = await new AxeBuilder({ page }).analyze();
  expect(results.violations).toEqual([]);
});
