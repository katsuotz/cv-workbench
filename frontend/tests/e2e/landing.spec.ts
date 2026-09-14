import AxeBuilder from '@axe-core/playwright';
import { expect, test } from '@playwright/test';

test('shows the CV Workbench landing page and routes visitors to the builder', async ({ page }) => {
  const apiRequests: string[] = [];
  page.on('request', (request) => {
    if (request.url().includes('/api/')) apiRequests.push(request.url());
  });

  await page.goto('/');
  await expect(
    page.getByRole('heading', { name: /Build an ATS-friendly CV\. See the result\./ })
  ).toBeVisible();
  await expect(
    page.getByText(
      'Create a clear, structured CV for applicant tracking systems, review the generated LaTeX, and download the finished PDF.'
    )
  ).toBeVisible();
  await expect(page.getByRole('region', { name: 'CV form builder' })).toHaveCount(0);
  await expect(page.getByRole('link', { name: 'Open builder' })).toHaveAttribute('href', '/app');
  await expect(page).toHaveTitle('CV Workbench — ATS-Friendly CV Builder');
  await expect(page.locator('html')).toHaveAttribute('lang', 'en');
  await expect(page.locator('meta[name="description"]')).toHaveAttribute(
    'content',
    /ATS-friendly CV.*structured sections.*exact LaTeX source.*polished PDF/
  );
  await expect(page.locator('link[rel="canonical"]')).toHaveAttribute('href', /\/$/);
  await expect(page.locator('meta[property="og:type"]')).toHaveAttribute('content', 'website');
  await expect(page.locator('meta[property="og:image"]')).toHaveAttribute('content', /https?:\/\//);
  await expect(page.locator('meta[name="twitter:card"]')).toHaveAttribute(
    'content',
    'summary_large_image'
  );
  const structuredData = JSON.parse(
    await page.locator('script[type="application/ld+json"]').evaluate((script) => script.innerHTML)
  );
  expect(structuredData['@graph']).toEqual(
    expect.arrayContaining([
      expect.objectContaining({ '@type': 'Organization' }),
      expect.objectContaining({ '@type': 'WebSite' }),
      expect.objectContaining({ '@type': 'WebApplication' })
    ])
  );
  expect(apiRequests).toEqual([]);

  const previews = page.getByRole('img', { name: /CV template preview/ });
  await expect(previews).toHaveCount(3);
  for (let index = 0; index < 3; index += 1) {
    expect(
      await previews.nth(index).evaluate((image) => (image as HTMLImageElement).naturalWidth)
    ).toBeGreaterThan(0);
  }
});

test('serves the Indonesian landing page with localized copy and SEO', async ({ page }) => {
  await page.goto('/id');
  await expect(page.locator('html')).toHaveAttribute('lang', 'id');
  await expect(
    page.getByRole('heading', { name: /Buat CV ramah ATS Anda\. Lihat hasilnya\./ })
  ).toBeVisible();
  await expect(
    page.getByText(
      'Buat CV yang jelas dan terstruktur untuk sistem pelacakan pelamar, tinjau LaTeX yang dihasilkan, lalu unduh PDF final.'
    )
  ).toBeVisible();
  await expect(page.getByRole('link', { name: 'Buka pembuat' })).toHaveAttribute('href', '/app');
  await expect(page).toHaveTitle('CV Workbench — Pembuat CV Ramah ATS');
  await expect(page.locator('link[rel="canonical"]')).toHaveAttribute('href', /\/id$/);
  await expect(page.locator('link[rel="alternate"][hreflang="en"]')).toHaveAttribute('href', /\/$/);
  await expect(page.locator('link[rel="alternate"][hreflang="id"]')).toHaveAttribute(
    'href',
    /\/id$/
  );
  await expect(page.locator('link[rel="alternate"][hreflang="x-default"]')).toHaveAttribute(
    'href',
    /\/$/
  );
  await expect(page.locator('meta[property="og:locale"]')).toHaveAttribute('content', 'id_ID');

  const structuredData = JSON.parse(
    await page.locator('script[type="application/ld+json"]').evaluate((script) => script.innerHTML)
  );
  expect(structuredData['@graph']).toEqual(
    expect.arrayContaining([
      expect.objectContaining({ '@type': 'WebSite', inLanguage: 'id-ID' }),
      expect.objectContaining({ '@type': 'WebApplication', inLanguage: 'id-ID' })
    ])
  );
});

test('publishes crawl and AI discovery files for the public site', async ({ request }) => {
  const robots = await request.get('/robots.txt');
  expect(robots.ok()).toBeTruthy();
  const robotsText = await robots.text();
  expect(robotsText).not.toContain('Disallow: /app');
  expect(robotsText).toContain('Disallow: /admin');
  expect(robotsText).toContain('Sitemap: https://cvworkbench.com/sitemap.xml');

  const sitemap = await request.get('/sitemap.xml');
  expect(sitemap.ok()).toBeTruthy();
  expect(await sitemap.text()).toContain('<loc>https://cvworkbench.com/</loc>');

  const llms = await request.get('/llms.txt');
  expect(llms.ok()).toBeTruthy();
  expect(await llms.text()).toContain('CV Workbench is an ATS-friendly web CV builder');
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
  await expect(page).toHaveTitle('CV Workbench — CV Builder');
  await expect(page.locator('meta[name="description"]')).toHaveAttribute(
    'content',
    /Build and refine your CV.*exact LaTeX source.*rendered PDF/
  );
  await expect(page.locator('meta[name="robots"]')).toHaveAttribute(
    'content',
    'noindex, nofollow, noarchive'
  );

  await page.goto('/admin');
  await expect(page).toHaveTitle('CV Workbench — Admin');
  await expect(page.locator('meta[name="description"]')).toHaveAttribute(
    'content',
    'Manage CV Workbench user accounts.'
  );
  await expect(page.locator('meta[name="robots"]')).toHaveAttribute(
    'content',
    'noindex, nofollow, noarchive'
  );
});

test('keeps the landing page usable without horizontal overflow on mobile', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/');
  await expect(
    page.getByRole('heading', { name: /Build an ATS-friendly CV\. See the result\./ })
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
    page.getByRole('heading', { name: /Build an ATS-friendly CV\. See the result\./ })
  ).toBeVisible();
  const results = await new AxeBuilder({ page }).analyze();
  expect(results.violations).toEqual([]);
});
