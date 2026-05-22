import { expect, test } from '@playwright/test';

test('view controls change the visible browser preview state', async ({ page }) => {
  await page.goto('/');

  await expect(page.getByRole('application', { name: 'RustFiles' })).toBeVisible();

  const toolbar = page.getByLabel('Toolbar');
  const fileBrowser = page.getByLabel('File browser');
  const sortKey = toolbar.getByLabel('Sort key');
  const sortDirection = toolbar.getByLabel('Sort direction');
  const filterKind = toolbar.getByLabel('Filter kind');
  const showHiddenFiles = toolbar.getByLabel('Show hidden files');
  const showFileExtensions = toolbar.getByLabel('Show file extensions');

  await expect(toolbar).toBeVisible();
  await expect(fileBrowser).toBeVisible();
  await expect(sortKey).toHaveValue('name');
  await expect(sortDirection).toHaveText('Ascending');
  await expect(filterKind).toHaveValue('all');
  await expect(showHiddenFiles).not.toBeChecked();
  await expect(showFileExtensions).toBeChecked();

  await expect(fileBrowser).toContainText('View settings: Name, Ascending, All, Hidden off, Extensions on');

  await sortKey.selectOption('modified');
  await sortDirection.click();
  await filterKind.selectOption('videos');
  await showHiddenFiles.check();
  await showFileExtensions.uncheck();

  await expect(sortKey).toHaveValue('modified');
  await expect(sortDirection).toHaveText('Descending');
  await expect(filterKind).toHaveValue('videos');
  await expect(showHiddenFiles).toBeChecked();
  await expect(showFileExtensions).not.toBeChecked();

  await expect(fileBrowser).toContainText('View settings: Modified, Descending, Videos, Hidden on, Extensions off');
});
