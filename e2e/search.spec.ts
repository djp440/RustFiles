import { expect, test } from '@playwright/test';

test.describe('search flow (browser preview)', () => {
  test('filters the current folder, supports recursive cancellation, and opens locations', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');

    await expect(page.getByRole('application', { name: 'RustFiles' })).toBeVisible();

    const toolbar = page.getByLabel('Toolbar');
    const fileBrowser = page.getByLabel('File browser');
    const pathInput = page.getByRole('textbox', { name: 'Path' });
    const searchInput = toolbar.getByRole('textbox', { name: 'Search files' });
    const recursiveToggle = toolbar.getByRole('checkbox', { name: 'Recursive search' });
    const clearButton = toolbar.getByRole('button', { name: 'Clear search' });
    const cancelButton = toolbar.getByRole('button', { name: 'Cancel search' });

    await pathInput.fill('C:\\Users\\demo');
    await pathInput.press('Enter');

    await expect(page.getByRole('listitem', { name: 'report-root.txt' })).toBeVisible();
    await expect(page.getByRole('listitem', { name: 'notes.txt' })).toBeVisible();

    await searchInput.fill('report');

    await expect(fileBrowser).toContainText('Current folder search');
    await expect(page.getByRole('listitem', { name: 'report-root.txt' })).toBeVisible();
    await expect(page.getByRole('listitem', { name: 'notes.txt' })).toHaveCount(0);

    await clearButton.click();

    await expect(searchInput).toHaveValue('');
    await expect(page.getByRole('listitem', { name: 'notes.txt' })).toBeVisible();

    await recursiveToggle.check();
    await searchInput.fill('search');

    await expect(fileBrowser).toContainText('Recursive search');
    await cancelButton.click();
    await expect(fileBrowser).toContainText(/cancelled|partial/i);

    await clearButton.click();
    await recursiveToggle.check();
    await searchInput.fill('search');

    const openLocationButton = page.getByRole('button', { name: 'Open location for search-notes.txt' });
    await expect(openLocationButton).toBeVisible();
    await openLocationButton.click();

    await expect(pathInput).toHaveValue('C:\\Users\\demo\\Projects\\RustFiles');

    await expect(openLocationButton).toBeVisible();
    await openLocationButton.click();
    await expect(page.getByRole('alert')).toContainText('项目已不存在或已移动');
    await expect(pathInput).toHaveValue('C:\\Users\\demo\\Projects\\RustFiles');
  });
});
