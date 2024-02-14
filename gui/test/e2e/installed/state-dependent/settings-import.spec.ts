import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { startInstalledApp } from '../installed-utils';
import { TestUtils } from '../../utils';
import { RoutePath } from '../../../../src/renderer/lib/routes';

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
});

test.afterAll(async () => {
  await page.close();
});

async function navigateToSettingsImport() {
  await util.waitForNavigation(async () => await page.click('button[aria-label="Settings"]'));
  await util.waitForNavigation(async () => await page.getByText('VPN settings').click());

  expect(
    await util.waitForNavigation(async () => await page.getByText('Server IP override').click())
  ).toEqual(RoutePath.settingsImport);

  const title = page.locator('h1')
  await expect(title).toHaveText('Server IP override');
}

test('App should display no overrides', async () => {
  await navigateToSettingsImport();
  await expect(page.getByTestId('status-title')).toHaveText('NO OVERRIDES IMPORTED');
  await expect(page.getByText('Clear all overrides')).toBeDisabled();
});

test('App should fail to import text', async () => {
  expect(
    await util.waitForNavigation(async () => await page.getByText('Import via text').click())
  ).toEqual(RoutePath.settingsTextImport);

  await page.locator('textarea').fill(INVALID_JSON);
  expect(
    await util.waitForNavigation(async () => await page.click('button[aria-label="Save"]'))
  ).toEqual(RoutePath.settingsImport);

  await expect(page.getByTestId('status-title')).toHaveText('NO OVERRIDES IMPORTED');
  await expect(page.getByTestId('status-subtitle')).toBeVisible();
  await expect(page.getByText('Clear all overrides')).toBeDisabled();
  await expect(page.getByTestId('status-subtitle')).not.toBeEmpty();
});

test('App should succeed to import text', async () => {
  expect(
    await util.waitForNavigation(async () => await page.getByText('Import via text').click())
  ).toEqual(RoutePath.settingsTextImport);

  const textarea = page.locator('textarea');
  await expect(textarea).toHaveValue(INVALID_JSON);
  await textarea.fill(VALID_JSON);
  expect(
    await util.waitForNavigation(async () => await page.click('button[aria-label="Save"]'))
  ).toEqual(RoutePath.settingsImport);

  await expect(page.getByTestId('status-title')).toHaveText('IMPORT SUCCESSFUL');
  await expect(page.getByTestId('status-subtitle')).toBeVisible();
  await expect(page.getByText('Clear all overrides')).toBeEnabled();
  await expect(page.getByTestId('status-subtitle')).not.toBeEmpty();

  await expect(page.getByTestId('status-title')).toHaveText('OVERRIDES ACTIVE');

  expect(
    await util.waitForNavigation(async () => await page.getByText('Import via text').click())
  ).toEqual(RoutePath.settingsTextImport);

  await expect(textarea).toHaveValue('');

  expect(
    await util.waitForNavigation(async () => await page.click('button[aria-label="Close"]'))
  ).toEqual(RoutePath.settingsImport);
});

test('App should show active overrides', async () => {
  expect(
    await util.waitForNavigation(async () => await page.click('button[aria-label="Back"]'))
  ).toEqual(RoutePath.vpnSettings);
  expect(
    await util.waitForNavigation(async () => await page.getByText('Server IP override').click())
  ).toEqual(RoutePath.settingsImport);

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
