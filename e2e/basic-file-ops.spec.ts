import { test, expect } from '@playwright/test';

async function navigateToDemoPath(page: any) {
  await page.goto('/');

  const pathInput = page.getByLabel('Path');
  await pathInput.fill('C:\\Users\\demo');
  await pathInput.press('Enter');

  await expect(page.getByRole('region', { name: 'File browser' })).toBeVisible();
  await expect(page.getByText(/report-root\.txt/)).toBeVisible({ timeout: 5000 });
}

test('file browser shows action buttons for basic operations', async ({ page }) => {
  await page.goto('/');

  const fileBrowser = page.getByRole('region', { name: 'File browser' });
  await expect(fileBrowser).toBeVisible();

  await expect(fileBrowser.getByRole('button', { name: /new folder/i })).toBeVisible();
  await expect(fileBrowser.getByRole('button', { name: /rename/i })).toBeVisible();
  await expect(fileBrowser.getByRole('button', { name: /delete/i })).toBeVisible();
  await expect(fileBrowser.getByRole('button', { name: /open/i })).toBeVisible();
  await expect(fileBrowser.getByRole('button', { name: /terminal/i })).toBeVisible();
  await expect(fileBrowser.getByRole('button', { name: /properties/i })).toBeVisible();
});

test('new folder shows input dialog', async ({ page }) => {
  await navigateToDemoPath(page);

  const fileBrowser = page.getByRole('region', { name: 'File browser' });
  await fileBrowser.getByRole('button', { name: /new folder/i }).click();

  const dialog = page.getByRole('dialog', { name: /new folder/i });
  await expect(dialog).toBeVisible();

  const input = dialog.getByRole('textbox', { name: /new folder name/i });
  await expect(input).toHaveValue('新建文件夹');

  await dialog.getByRole('button', { name: /取消/i }).click();
  await expect(page.getByRole('dialog', { name: /new folder/i })).not.toBeVisible();
});

test('inline rename shows input and handles Enter commit', async ({ page }) => {
  await navigateToDemoPath(page);

  const fileBrowser = page.getByRole('region', { name: 'File browser' });

  const firstEntry = fileBrowser.getByText(/report-root\.txt/).first();
  await firstEntry.click();

  await fileBrowser.getByRole('button', { name: /rename/i }).click();

  const renameInput = fileBrowser.getByRole('textbox', { name: /rename input/i });
  await expect(renameInput).toBeVisible();
  await expect(renameInput).toHaveValue('report-root.txt');

  await renameInput.fill('renamed-file.txt');
  await renameInput.press('Enter');

  await expect(fileBrowser).toBeVisible();
});

test('inline rename cancels on Escape', async ({ page }) => {
  await navigateToDemoPath(page);

  const fileBrowser = page.getByRole('region', { name: 'File browser' });

  const firstEntry = fileBrowser.getByText(/report-root\.txt/).first();
  await firstEntry.click();

  await fileBrowser.getByRole('button', { name: /rename/i }).click();

  const renameInput = fileBrowser.getByRole('textbox', { name: /rename input/i });
  await expect(renameInput).toBeVisible();

  await renameInput.press('Escape');

  await expect(fileBrowser.getByRole('textbox', { name: /rename input/i })).not.toBeVisible();
});

test('inline rename shows error for illegal characters', async ({ page }) => {
  await navigateToDemoPath(page);

  const fileBrowser = page.getByRole('region', { name: 'File browser' });

  const firstEntry = fileBrowser.getByText(/report-root\.txt/).first();
  await firstEntry.click();

  await fileBrowser.getByRole('button', { name: /rename/i }).click();

  const renameInput = fileBrowser.getByRole('textbox', { name: /rename input/i });
  await renameInput.fill('bad<name.txt');
  await renameInput.press('Enter');

  await expect(fileBrowser.getByText(/非法字符/)).toBeVisible();
});

test('inline rename shows error for reserved name', async ({ page }) => {
  await navigateToDemoPath(page);

  const fileBrowser = page.getByRole('region', { name: 'File browser' });

  const firstEntry = fileBrowser.getByText(/report-root\.txt/).first();
  await firstEntry.click();

  await fileBrowser.getByRole('button', { name: /rename/i }).click();

  const renameInput = fileBrowser.getByRole('textbox', { name: /rename input/i });
  await renameInput.fill('CON');
  await renameInput.press('Enter');

  await expect(fileBrowser.getByText(/保留名/)).toBeVisible();
});

test('delete to recycle bin shows confirmation and proceeds', async ({ page }) => {
  await navigateToDemoPath(page);

  const fileBrowser = page.getByRole('region', { name: 'File browser' });

  const firstEntry = fileBrowser.getByText(/report-root\.txt/).first();
  await firstEntry.click();

  await fileBrowser.getByRole('button', { name: /delete/i }).click();

  const confirmDialog = page.getByRole('dialog', { name: /delete/i });
  await expect(confirmDialog).toBeVisible();
  await expect(confirmDialog.getByText(/移到回收站/i)).toBeVisible();

  await confirmDialog.getByRole('button', { name: /移到回收站/i }).click();

  await expect(fileBrowser).toBeVisible();
});

test('delete permanently requires confirmation and cancel is default focus', async ({ page }) => {
  await navigateToDemoPath(page);

  const fileBrowser = page.getByRole('region', { name: 'File browser' });

  const firstEntry = fileBrowser.getByText(/report-root\.txt/).first();
  await firstEntry.click();

  await fileBrowser.getByRole('button', { name: /delete/i }).click();

  const confirmDialog = page.getByRole('dialog', { name: /delete/i });
  await expect(confirmDialog).toBeVisible();

  const cancelButton = confirmDialog.getByRole('button', { name: /取消/i });
  await expect(cancelButton).toBeVisible();
  await expect(cancelButton).toBeFocused();
});

test('delete confirm dialog can be cancelled', async ({ page }) => {
  await navigateToDemoPath(page);

  const fileBrowser = page.getByRole('region', { name: 'File browser' });

  const firstEntry = fileBrowser.getByText(/report-root\.txt/).first();
  await firstEntry.click();

  await fileBrowser.getByRole('button', { name: /delete/i }).click();

  const confirmDialog = page.getByRole('dialog', { name: /delete/i });
  await expect(confirmDialog).toBeVisible();

  await confirmDialog.getByRole('button', { name: /取消/i }).click();

  await expect(page.getByRole('dialog', { name: /delete/i })).not.toBeVisible();
});

test('open file shows preview feedback in browser mode', async ({ page }) => {
  await navigateToDemoPath(page);

  const fileBrowser = page.getByRole('region', { name: 'File browser' });

  const firstEntry = fileBrowser.getByText(/report-root\.txt/).first();
  await firstEntry.click();

  await fileBrowser.getByRole('button', { name: /open/i }).click();

  await expect(fileBrowser.getByText(/预览模式/)).toBeVisible();
});

test('open terminal shows preview feedback in browser mode', async ({ page }) => {
  await navigateToDemoPath(page);

  const fileBrowser = page.getByRole('region', { name: 'File browser' });
  await fileBrowser.getByRole('button', { name: /terminal/i }).click();

  await expect(fileBrowser.getByText(/终端/).first()).toBeVisible();
});

test('properties dialog shows file info', async ({ page }) => {
  await navigateToDemoPath(page);

  const fileBrowser = page.getByRole('region', { name: 'File browser' });

  const firstEntry = fileBrowser.getByText(/report-root\.txt/).first();
  await firstEntry.click();

  await fileBrowser.getByRole('button', { name: /properties/i }).click();

  const propsDialog = page.getByRole('dialog', { name: /properties/i });
  await expect(propsDialog).toBeVisible();
  await expect(propsDialog.getByText(/report-root\.txt/).first()).toBeVisible();

  await propsDialog.getByRole('button', { name: /close/i }).click();

  await expect(page.getByRole('dialog', { name: /properties/i })).not.toBeVisible();
});
