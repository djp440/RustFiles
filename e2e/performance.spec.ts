import { expect, test } from '@playwright/test';

test.describe('Virtual list performance (browser preview)', () => {
  test('view mode buttons are present and clickable', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');

    const listButton = page.getByRole('button', { name: /list view/i });
    await expect(listButton).toBeVisible();
    expect(listButton).toHaveAttribute('aria-pressed', 'true');

    const iconButton = page.getByRole('button', { name: /icon view/i });
    await expect(iconButton).toBeVisible();

    const detailsButton = page.getByRole('button', { name: /details view/i });
    await expect(detailsButton).toBeVisible();

    await iconButton.click();
    await expect(iconButton).toHaveAttribute('aria-pressed', 'true');
    await expect(listButton).toHaveAttribute('aria-pressed', 'false');

    await detailsButton.click();
    await expect(detailsButton).toHaveAttribute('aria-pressed', 'true');
    await expect(iconButton).toHaveAttribute('aria-pressed', 'false');
  });

  test('navigating to a path does not cause crash', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');

    const pathInput = page.getByRole('textbox', { name: 'Path' });
    await expect(pathInput).toBeVisible();

    await pathInput.fill('C:\\Users');
    await pathInput.press('Enter');

    await expect(pathInput).toHaveValue('C:\\Users');
    await expect(page.locator('body')).toBeVisible();
  });

  test('sidebar navigation is functional', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');

    const sidebar = page.getByLabel('Sidebar');
    await expect(sidebar).toBeVisible();

    await expect(sidebar.getByRole('button', { name: 'This PC' })).toBeVisible();
    await expect(sidebar.getByRole('button', { name: 'Desktop' })).toBeVisible();
  });
});
