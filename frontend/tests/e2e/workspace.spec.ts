import { expect, test } from '@playwright/test';
import { readFile } from 'node:fs/promises';

test.beforeEach(async ({ page }) => {
  await page.goto('/app');
  await page.reload();
  await expect(page.getByRole('button', { name: 'Preview', exact: true })).toBeEnabled();
});

test('starts in focused intake, validates, and reveals the preview workspace on request', async ({
  page
}) => {
  await expect(page.getByRole('button', { name: 'Download PDF' })).toHaveCount(0);
  await expect(page.getByRole('region', { name: 'Rendered preview' })).toHaveCount(0);
  await expect(page.getByRole('button', { name: 'Preview', exact: true })).toBeEnabled();
  await page.getByRole('button', { name: 'Preview', exact: true }).click();
  await expect(page.getByRole('alert')).toContainText('Full name is required.');
  await expect(page.getByRole('region', { name: 'Rendered preview' })).toHaveCount(0);
  await expect(page.getByLabel(/Full name/)).toBeFocused();
  await page.getByLabel(/Full name/).fill('Ada Lovelace');
  await page.getByLabel('Email').fill('ada@example.com');
  await page.getByRole('button', { name: 'Preview', exact: true }).click();
  await expect(
    page.getByRole('region', { name: 'Rendered preview' }).getByLabel('PDF page 1')
  ).toBeVisible();
  await expect(page.getByRole('region', { name: 'Rendered preview' })).toBeVisible();
  await expect(page.locator('.proof-status')).toHaveCount(0);
  await expect(page.getByRole('button', { name: 'Refresh preview' })).toBeEnabled();
  await page.getByRole('button', { name: 'Refresh preview' }).click();
  await expect(
    page.getByRole('region', { name: 'Rendered preview' }).getByLabel('PDF page 1')
  ).toBeVisible();
  await page.getByLabel(/Full name/).fill('Ada King');
});

test('downloads the backend artifact bytes with the CV filename', async ({ page }) => {
  await page.getByLabel(/Full name/).fill('Ada Lovelace');
  await page.getByLabel('Email').fill('ada@example.com');
  await page.getByRole('button', { name: 'Preview', exact: true }).click();
  await expect(
    page.getByRole('region', { name: 'Rendered preview' }).getByLabel('PDF page 1')
  ).toBeVisible();

  const downloadPromise = page.waitForEvent('download');
  await page.getByRole('button', { name: 'Download PDF' }).click();
  const download = await downloadPromise;
  expect(download.suggestedFilename()).toBe('ada-lovelace-cv.pdf');
  const path = await download.path();
  expect(path).not.toBeNull();
  const bytes = await readFile(path!);
  expect(bytes.subarray(0, 4).toString('ascii')).toBe('%PDF');
});

test('keeps the last backend PDF visible when compilation fails', async ({ page }) => {
  await page.getByLabel(/Full name/).fill('Ada Lovelace');
  await page.getByLabel('Email').fill('ada@example.com');
  await page.getByRole('button', { name: 'Preview', exact: true }).click();
  await expect(
    page.getByRole('region', { name: 'Rendered preview' }).getByLabel('PDF page 1')
  ).toBeVisible();

  await page.getByLabel(/Full name/).fill('E2E COMPILE FAILURE');
  await page.getByRole('button', { name: 'Preview', exact: true }).click();
  await expect(page.getByRole('region', { name: 'Compiler notes' })).toContainText(
    'Fixture compiler rejected this source.'
  );
  await expect(
    page.getByRole('region', { name: 'Rendered preview' }).getByLabel('PDF page 1')
  ).toBeVisible();
});

test('preserves source and preview when backend rendering fails', async ({ page }) => {
  await page.getByLabel(/Full name/).fill('Ada Lovelace');
  await page.getByLabel('Email').fill('ada@example.com');
  await page.getByRole('button', { name: 'Preview', exact: true }).click();
  await expect(
    page.getByRole('region', { name: 'Rendered preview' }).getByLabel('PDF page 1')
  ).toBeVisible();

  await page.getByLabel(/Full name/).fill('E2E RENDER FAILURE');
  await page.getByRole('button', { name: 'Preview', exact: true }).click();
  await expect(page.getByText('Fixture renderer rejected this CV.')).toBeVisible();
  await expect(
    page.getByRole('region', { name: 'Rendered preview' }).getByLabel('PDF page 1')
  ).toBeVisible();
  await page.getByRole('button', { name: 'Source' }).click();
  await expect(page.locator('.source-code')).toContainText('Generated source for Ada Lovelace');
});

test('selects a template and persists it', async ({ page }) => {
  await page.getByLabel('Use Compact signal template').check();
  await expect(page.getByRole('radio', { name: 'Use Compact signal template' })).toBeChecked();
  await page.getByLabel(/Full name/).fill('Ada Lovelace');
  await page.getByLabel('Email').fill('ada@example.com');
  await page.getByRole('button', { name: 'Preview', exact: true }).click();
  await expect(
    page.getByRole('region', { name: 'Rendered preview' }).getByLabel('PDF page 1')
  ).toBeVisible();
  await page.getByLabel('Use Modern hierarchy template').check();
  await page.waitForTimeout(900);
  await page.reload();
  await expect(page.getByRole('radio', { name: 'Use Modern hierarchy template' })).toBeChecked();
});

test('opens a larger example preview from the template picker', async ({ page }) => {
  const previewButton = page.getByRole('button', {
    name: 'View larger preview of Editorial dossier'
  });
  await expect(previewButton).toBeVisible();
  await previewButton.click();

  const dialog = page.getByRole('dialog', { name: 'Editorial dossier' });
  await expect(dialog).toBeVisible();
  await expect(dialog.getByLabel('PDF page 1')).toBeVisible();
  await dialog.getByRole('button', { name: 'Close' }).click();
  await expect(dialog).toBeHidden();
});

test('opens the preview workspace with actionable diagnostics on a first-run failure', async ({
  page
}) => {
  await page.getByLabel(/Full name/).fill('E2E COMPILE FAILURE');
  await page.getByLabel('Email').fill('ada@example.com');
  await page.getByRole('button', { name: 'Preview', exact: true }).click();
  await expect(page.getByRole('region', { name: 'Rendered preview' })).toBeVisible();
  await expect(page.getByRole('region', { name: 'Compiler notes' })).toContainText(
    'Fixture compiler rejected this source.'
  );
  await expect(page.getByText('Preview unavailable')).toBeVisible();
  await page.getByRole('button', { name: /Fixture compiler rejected this source/ }).click();
  await expect(page.getByRole('heading', { name: 'LaTeX source' })).toBeVisible();
  await expect(page.locator('.source-code .cm-editor')).toBeVisible();
  await expect(page.locator('.source-code .cm-content')).toHaveAttribute(
    'contenteditable',
    'false'
  );
  await expect(page.locator('.source-code .cm-editor')).toHaveCSS('color', 'rgb(213, 226, 238)');
});

test('moves between form sections and exposes exact source actions', async ({ page }) => {
  await page.getByLabel(/Full name/).fill('Ada Lovelace');
  await page.getByLabel('Email').fill('ada@example.com');
  await page.getByRole('button', { name: 'Preview', exact: true }).click();
  await page.getByRole('button', { name: 'Source' }).click();
  await expect(page.getByRole('heading', { name: 'LaTeX source' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Download .tex' })).toBeEnabled();
});

test('opens and closes a full-page preview', async ({ page }) => {
  await page.getByLabel(/Full name/).fill('Ada Lovelace');
  await page.getByLabel('Email').fill('ada@example.com');
  await page.getByRole('button', { name: 'Preview', exact: true }).click();
  await expect(page.getByRole('button', { name: 'Full page' })).toBeVisible();
  await page.getByRole('button', { name: 'Full page' }).click();
  const dialog = page.getByRole('dialog', { name: 'Full-page preview' });
  await expect(dialog).toBeVisible();
  await expect(dialog.getByLabel('PDF page 1')).toBeVisible();
  await dialog.getByRole('button', { name: 'Close' }).click();
  await expect(dialog).toBeHidden();
});

test('keeps form and preview navigation usable on mobile', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await expect(page.getByRole('button', { name: 'Preview', exact: true })).toBeVisible();
  await page.getByLabel(/Full name/).fill('Ada Lovelace');
  await page.getByLabel('Email').fill('ada@example.com');
  await page.getByRole('button', { name: 'Preview', exact: true }).click();
  await expect(page.getByRole('region', { name: 'Rendered preview' })).toBeVisible();
  await expect(
    page.getByRole('region', { name: 'Rendered preview' }).getByLabel('PDF page 1')
  ).toBeVisible();
  await page.getByRole('button', { name: 'Form', exact: true }).click();
  await expect(page.getByRole('region', { name: 'CV form builder' })).toBeVisible();
  await page.getByRole('button', { name: /Next/ }).click();
  await expect(page.getByRole('heading', { name: 'Experience' })).toBeVisible();
});

test('switches to the single-pane layout before desktop columns can overflow', async ({ page }) => {
  await page.setViewportSize({ width: 820, height: 900 });
  await page.getByLabel(/Full name/).fill('Ada Lovelace');
  await page.getByLabel('Email').fill('ada@example.com');
  await page.getByRole('button', { name: 'Preview', exact: true }).click();
  await expect(page.getByRole('button', { name: 'Preview', exact: true })).toBeVisible();
  expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBeLessThanOrEqual(820);
  await page.getByRole('button', { name: 'Source' }).click();
  await expect(page.getByRole('heading', { name: 'LaTeX source' })).toBeVisible();
});

test('fills the desktop preview workspace below the header', async ({ page }) => {
  await page.setViewportSize({ width: 2048, height: 1080 });
  await page.getByLabel(/Full name/).fill('Ada Lovelace');
  await page.getByLabel('Email').fill('ada@example.com');
  await page.getByRole('button', { name: 'Preview', exact: true }).click();
  await expect(
    page.getByRole('region', { name: 'Rendered preview' }).getByLabel('PDF page 1')
  ).toBeVisible();

  const layout = await page.evaluate(() => {
    const content = document.querySelector<HTMLElement>('.workspace-content');
    const form = document.querySelector<HTMLElement>('.form-panel');
    const preview = document.querySelector<HTMLElement>('.preview-panel');
    return {
      contentHeight: content?.getBoundingClientRect().height ?? 0,
      formHeight: form?.getBoundingClientRect().height ?? 0,
      previewHeight: preview?.getBoundingClientRect().height ?? 0
    };
  });

  expect(layout.contentHeight).toBeGreaterThan(900);
  expect(layout.formHeight).toBeGreaterThan(900);
  expect(layout.previewHeight).toBeGreaterThan(900);
});

test('reopens saved generated work directly in the preview workspace', async ({ page }) => {
  await page.getByLabel(/Full name/).fill('Ada Lovelace');
  await page.getByLabel('Email').fill('ada@example.com');
  await page.getByRole('button', { name: 'Preview', exact: true }).click();
  await expect(
    page.getByRole('region', { name: 'Rendered preview' }).getByLabel('PDF page 1')
  ).toBeVisible();
  await page.reload();
  await expect(page.getByRole('region', { name: 'Rendered preview' })).toBeVisible();
  await expect(
    page.getByRole('region', { name: 'Rendered preview' }).getByLabel('PDF page 1')
  ).toBeVisible();
});

test('continues editing when remote autosave is unavailable', async ({ page }) => {
  await page.route('**/api/v1/cv/session', async (route) => {
    if (route.request().method() === 'PUT') {
      await route.fulfill({
        status: 503,
        contentType: 'application/json',
        body: JSON.stringify({ code: 'service_unavailable', message: 'Autosave is offline.' })
      });
      return;
    }
    await route.continue();
  });
  await page.reload();
  await expect(page.getByRole('button', { name: 'Preview', exact: true })).toBeEnabled();
  await page.getByLabel(/Full name/).fill('Ada Lovelace');
  await expect(page.getByText(/Could not save this draft/)).toBeVisible();
  await page.getByLabel('Email').fill('ada@example.com');
  await page.getByRole('button', { name: 'Preview', exact: true }).click();
  await expect(
    page.getByRole('region', { name: 'Rendered preview' }).getByLabel('PDF page 1')
  ).toBeVisible();
});
