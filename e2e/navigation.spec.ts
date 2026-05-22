import { expect, test } from '@playwright/test';

test('navigation shell supports basic user navigation flow', async ({ page }) => {
  await page.goto('/');

  await expect(page.getByRole('application', { name: 'RustFiles' })).toBeVisible();

  const pathInput = page.getByRole('textbox', { name: 'Path' });
  const backButton = page.getByRole('button', { name: 'Back' });
  const forwardButton = page.getByRole('button', { name: 'Forward' });
  const sidebar = page.getByLabel('Sidebar');
  const breadcrumb = page.getByLabel('Breadcrumb');

  await expect(pathInput).toHaveValue('This PC');
  await expect(page.getByText('Current location:').locator('..')).toContainText('This PC');
  await expect(backButton).toBeDisabled();
  await expect(forwardButton).toBeDisabled();

  await expect(sidebar.getByRole('button', { name: 'This PC' })).toBeVisible();
  await expect(sidebar.getByRole('button', { name: 'Desktop' })).toBeVisible();
  await expect(sidebar.getByRole('button', { name: 'Downloads' })).toBeVisible();
  await expect(sidebar.getByRole('button', { name: 'Documents' })).toBeVisible();
  await expect(sidebar.getByRole('button', { name: 'Pictures' })).toBeVisible();
  await expect(sidebar.getByRole('button', { name: 'Videos' })).toBeVisible();
  await expect(sidebar.getByRole('button', { name: 'Music' })).toBeVisible();
  await expect(
    sidebar.getByText('Browser preview mode: desktop runtime features are limited.'),
  ).toBeVisible();
  await expect(sidebar.getByText('No drives available')).toHaveCount(0);
  await expect(
    page.getByText('This location is a preview entry. Browse real folders in the desktop app runtime.'),
  ).toBeVisible();

  await pathInput.fill('C:\\Users');
  await pathInput.press('Enter');

  await expect(pathInput).toHaveValue('C:\\Users');
  await expect(breadcrumb.getByRole('button', { name: 'C:' })).toBeVisible();
  await expect(breadcrumb.getByRole('button', { name: 'Users' })).toBeVisible();
  await expect(backButton).toBeEnabled();
  await expect(forwardButton).toBeDisabled();

  await backButton.click();

  await expect(pathInput).toHaveValue('This PC');
  await expect(breadcrumb.getByRole('button', { name: 'This PC' })).toBeVisible();
  await expect(backButton).toBeDisabled();
  await expect(forwardButton).toBeEnabled();

  await forwardButton.click();

  await expect(pathInput).toHaveValue('C:\\Users');
  await expect(breadcrumb.getByRole('button', { name: 'Users' })).toBeVisible();
  await expect(backButton).toBeEnabled();
});
