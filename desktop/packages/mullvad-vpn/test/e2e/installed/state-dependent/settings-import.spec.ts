import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { RoutePath } from '../../../../src/renderer/lib/routes';
import { TestUtils } from '../../utils';
import { startInstalledApp } from '../installed-utils';

const INVALID_JSON = 'invalid json';
const VALID_JSON = `
{
  "relay_overrides": [
    {
      "hostname": "se-got-wg-001",
      "ipv4_addr_in": "127.0.0.1"
    }
  ]
}
`;

// This test expects the daemon to be logged in.

let page: Page;
let util: TestUtils;

test.beforeAll(async () => {
  ({ page, util } = await startInstalledApp());
  await util.waitForRoute(RoutePath.main);
});

test.afterAll(async () => {
  await page.close();
});

async function navigateToSettingsImport() {
  await page.click('button[aria-label="Settings"]');
  await util.waitForRoute(RoutePath.settings);

  await page.getByRole('button', { name: 'VPN settings' }).click();
  await util.waitForRoute(RoutePath.vpnSettings);

  await page.getByRole('button', { name: 'Server IP override' }).click();
  await util.waitForRoute(RoutePath.settingsImport);

  const title = page.locator('h1');
  await expect(title).toHaveText('Server IP override');
}

test('App should display no overrides', async () => {
  await navigateToSettingsImport();
  await expect(page.getByTestId('status-title')).toHaveText('NO OVERRIDES IMPORTED');
  await expect(page.getByText('Clear all overrides')).toBeDisabled();
});

test('App should fail to import text', async () => {
  await page.getByRole('button', { name: 'Import via text' }).click();
  await util.waitForRoute(RoutePath.settingsTextImport);

  await page.locator('textarea').fill(INVALID_JSON);
  await page.click('button[aria-label="Save"]');
  await util.waitForRoute(RoutePath.settingsImport);

  await expect(page.getByTestId('status-title')).toHaveText('NO OVERRIDES IMPORTED');
  await expect(page.getByTestId('status-subtitle')).toBeVisible();
  await expect(page.getByText('Clear all overrides')).toBeDisabled();
  await expect(page.getByTestId('status-subtitle')).not.toBeEmpty();
});

test('App should succeed to import text', async () => {
  await page.getByRole('button', { name: 'Import via text' }).click();
  await util.waitForRoute(RoutePath.settingsTextImport);

  const textarea = page.locator('textarea');
  await expect(textarea).toHaveValue(INVALID_JSON);
  await textarea.fill(VALID_JSON);

  await page.click('button[aria-label="Save"]');
  await util.waitForRoute(RoutePath.settingsImport);

  await expect(page.getByTestId('status-title')).toHaveText('IMPORT SUCCESSFUL');
  await expect(page.getByTestId('status-subtitle')).toBeVisible();
  await expect(page.getByText('Clear all overrides')).toBeEnabled();
  await expect(page.getByTestId('status-subtitle')).not.toBeEmpty();

  await expect(page.getByTestId('status-title')).toHaveText('OVERRIDES ACTIVE');

  await page.getByRole('button', { name: 'Import via text' }).click();
  await util.waitForRoute(RoutePath.settingsTextImport);

  await expect(textarea).toHaveValue('');

  await page.click('button[aria-label="Close"]');
  await util.waitForRoute(RoutePath.settingsImport);
});

test('App should show active overrides', async () => {
  await page.click('button[aria-label="Back"]');
  await util.waitForRoute(RoutePath.vpnSettings);

  await page.getByRole('button', { name: 'Server IP override' }).click();
  await util.waitForRoute(RoutePath.settingsImport);

  await expect(page.getByTestId('status-title')).toHaveText('OVERRIDES ACTIVE');
  await expect(page.getByText('Clear all overrides')).toBeEnabled();
});

test('App should clear overrides', async () => {
  await page.getByText('Clear all overrides').click();
  await expect(page.getByText('Clear all overrides?')).toBeVisible();

  await page.getByText(/^Clear$/).click();
  await expect(page.getByTestId('status-title')).toHaveText('NO OVERRIDES IMPORTED');
  await expect(page.getByText(/Clear all overrides$/)).toBeDisabled();
});
