import { expect, test } from '@playwright/test';

test.describe('Glass Readability and View Switching', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to the app in browser preview mode
    await page.goto('http://127.0.0.1:1420');
  });

  test('switches view mode button states', async ({ page }) => {
    const fileBrowser = page.getByLabel('File browser');
    await expect(fileBrowser).toBeVisible();

    // Default button should be pressed
    const listBtn = page.getByRole('button', { name: 'List' });
    const iconBtn = page.getByRole('button', { name: 'Icon' });
    const detailsBtn = page.getByRole('button', { name: 'Details' });

    await expect(listBtn).toHaveAttribute('aria-pressed', 'true');
    await expect(iconBtn).toHaveAttribute('aria-pressed', 'false');
    await expect(detailsBtn).toHaveAttribute('aria-pressed', 'false');

    // Switch to icon view
    await iconBtn.click();
    await expect(listBtn).toHaveAttribute('aria-pressed', 'false');
    await expect(iconBtn).toHaveAttribute('aria-pressed', 'true');

    // Switch to details view
    await detailsBtn.click();
    await expect(iconBtn).toHaveAttribute('aria-pressed', 'false');
    await expect(detailsBtn).toHaveAttribute('aria-pressed', 'true');
  });

  test('applies glass surface tokens and variants', async ({ page }) => {
    const fileBrowser = page.getByLabel('File browser');
    await expect(fileBrowser).toHaveAttribute('data-surface-variant', 'content');
    
    // Check computed styles for tokens
    const styles = await fileBrowser.evaluate((el) => {
      const computed = window.getComputedStyle(el);
      return {
        background: computed.backgroundColor,
        borderRadius: computed.borderRadius,
        border: computed.border,
      };
    });
    
    // Should use var(--surface-content) which is rgba(35, 35, 35, 0.4)
    expect(styles.background).toContain('rgba(35, 35, 35, 0.4)');
    expect(styles.borderRadius).toBe('10px'); // var(--radius-md)
  });

  test('maintains readability in browser fallback', async ({ page }) => {
    // The browser fallback message should be visible and readable
    const fallbackText = page.getByText('This location is a preview entry. Browse real folders in the desktop app runtime.');
    await expect(fallbackText).toBeVisible();
    
    // Check text colors from tokens
    const color = await fallbackText.evaluate((el) => window.getComputedStyle(el).color);
    // Should be var(--text-secondary) -> rgba(255, 255, 255, 0.7)
    expect(color).toContain('rgba(255, 255, 255, 0.7)');
  });
});
