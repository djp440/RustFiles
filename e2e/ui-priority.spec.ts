import { expect, test } from '@playwright/test';

test.describe('UI priority scheduler reporting (browser preview)', () => {
  test('records browser-preview viewport and interaction evidence for real navigation controls', async ({
    page,
  }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');

    await expect(page.getByRole('application', { name: 'RustFiles' })).toBeVisible();
    await expect(page.getByLabel('Sidebar')).toBeVisible();
    await expect(page.getByLabel('Navigation')).toBeVisible();
    await expect(page.getByLabel('File browser')).toBeVisible();

    await expect(page.getByText('Browser preview mode: desktop runtime features are limited.')).toBeVisible();

    const debugSnapshot = async () =>
      page.evaluate(() => (window as Window & { __RUSTFILES_SCHEDULER_DEBUG__?: unknown }).__RUSTFILES_SCHEDULER_DEBUG__);

    await expect
      .poll(async () => {
        const state = (await debugSnapshot()) as
          | {
              reports?: Array<{
                report_kind: string;
                ack: {
                  interaction_latency_ms: number | null;
                  frame_budget_degraded: boolean;
                };
              }>;
            }
          | undefined;

        return state?.reports?.length ?? 0;
      })
      .toBeGreaterThan(0);

    const pathInput = page.getByRole('textbox', { name: 'Path' });
    await pathInput.fill('C:\\Users\\demo');
    await pathInput.press('Enter');

    const breadcrumb = page.getByLabel('Breadcrumb');
    await expect(breadcrumb.getByRole('button', { name: 'Users' })).toBeVisible();
    await breadcrumb.getByRole('button', { name: 'Users' }).click();
    await expect(pathInput).toHaveValue('C:\\Users');

    const listViewButton = page.getByRole('button', { name: 'List view' });
    const detailsViewButton = page.getByRole('button', { name: 'Details view' });
    await expect(listViewButton).toBeVisible();
    await expect(detailsViewButton).toBeVisible();
    await detailsViewButton.click();
    await expect(detailsViewButton).toHaveAttribute('aria-pressed', 'true');
    await listViewButton.click();
    await expect(listViewButton).toHaveAttribute('aria-pressed', 'true');

    const sidebar = page.getByLabel('Sidebar');
    await sidebar.getByRole('button', { name: 'Desktop' }).click();
    await expect(pathInput).toHaveValue('Desktop');

    const finalState = (await debugSnapshot()) as {
      reports?: Array<{ report_kind: string; ack: { interaction_latency_ms: number | null; frame_budget_degraded: boolean; interaction_epoch: number } }>;
      latest_viewport_ack?: { interaction_latency_ms: number | null; frame_budget_degraded: boolean; visible_range: unknown };
      latest_interaction_ack?: { interaction_latency_ms: number | null; frame_budget_degraded: boolean; interaction_epoch: number };
    };

    expect(finalState?.reports?.some((report) => report.report_kind === 'viewport')).toBeTruthy();
    expect(finalState?.reports?.some((report) => report.report_kind === 'interaction')).toBeTruthy();
    expect(finalState?.latest_viewport_ack?.interaction_latency_ms).not.toBeNull();
    expect(finalState?.latest_viewport_ack?.frame_budget_degraded).not.toBeUndefined();
    expect(finalState?.latest_interaction_ack?.interaction_latency_ms).not.toBeNull();
    expect(finalState?.latest_interaction_ack?.interaction_epoch).toBeGreaterThan(0);

    await expect(page.getByRole('button', { name: 'Details view' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'List view' })).toBeVisible();
  });
});
