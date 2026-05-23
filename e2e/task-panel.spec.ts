import { test, expect } from '@playwright/test';

test('task panel stays compact and expandable', async ({ page }) => {
  await page.goto('/');

  const taskPanel = page.getByRole('region', { name: 'Task panel' });
  await expect(taskPanel).toBeVisible();
  await expect(taskPanel.getByText(/active task\(s\)/i)).toBeVisible();
  await expect(taskPanel.getByText(/copying 2 items/i)).toBeVisible();

  await taskPanel.getByRole('button', { name: /tasks/i }).click();
  await expect(taskPanel.getByText(/copying 2 items/i)).toBeVisible();
  await expect(taskPanel.getByText(/waiting for conflict decision/i).first()).toBeVisible();
  await expect(taskPanel.getByRole('button', { name: /collapse tasks/i })).toBeVisible();
});
