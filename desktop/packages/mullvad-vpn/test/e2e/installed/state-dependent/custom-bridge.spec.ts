import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { colors } from '../../../../src/renderer/lib/foundations';
import { RoutePath } from '../../../../src/renderer/lib/routes';
import { TestUtils } from '../../utils';
import { startInstalledApp } from '../installed-utils';

// This test expects the daemon to be logged in and not have a custom bridge configured.
// Env parameters:
//   `SHADOWSOCKS_SERVER_IP`
//   `SHADOWSOCKS_SERVER_PORT`
//   `SHADOWSOCKS_SERVER_CIPHER`
//   `SHADOWSOCKS_SERVER_PASSWORD`

let page: Page;
let util: TestUtils;

test.beforeAll(async () => {
  ({ page, util } = await startInstalledApp());
  await util.waitForRoute(RoutePath.main);
});

test.afterAll(async () => {
  await page.close();
});

test('App should enable bridge mode', async () => {
  await page.click('button[aria-label="Settings"]');
  await util.waitForRoute(RoutePath.settings);

  await page.getByText('VPN settings').click();
  await util.waitForRoute(RoutePath.vpnSettings);

  await page.getByRole('option', { name: 'OpenVPN' }).click();
  await page.getByText('OpenVPN settings').click();
  await util.waitForRoute(RoutePath.openVpnSettings);

  const bridgeModeOnButton = page.getByTestId('bridge-mode-on');
  await bridgeModeOnButton.click();
  await expect(bridgeModeOnButton).toHaveAttribute('aria-selected', 'true');

  await page.click('button[aria-label="Back"]');
  await util.waitForRoute(RoutePath.vpnSettings);

  await page.click('button[aria-label="Back"]');
  await util.waitForRoute(RoutePath.settings);

  await page.click('button[aria-label="Close"]');
  await util.waitForRoute(RoutePath.main);
});

test('App display disabled custom bridge', async () => {
  await page.click('button[aria-label^="Select location"]');
  await util.waitForRoute(RoutePath.selectLocation);

  const title = page.locator('h1');
  await expect(title).toHaveText('Select location');

  await page.getByText(/^Entry$/).click();

  const customBridgeButton = page.locator('button:has-text("Custom bridge")');
  await expect(customBridgeButton).toBeDisabled();
});

test('App should add new custom bridge', async () => {
  await page.click('button[aria-label="Add new custom bridge"]');
  await util.waitForRoute(RoutePath.editCustomBridge);

  const title = page.locator('h1');
  await expect(title).toHaveText('Add custom bridge');

  const inputs = page.locator('input');
  const addButton = page.locator('button:has-text("Add")');
  await expect(addButton).toBeVisible();
  await expect(addButton).toBeDisabled();

  await inputs.first().fill(process.env.SHADOWSOCKS_SERVER_IP!);
  await expect(addButton).toBeDisabled();

  await inputs.nth(1).fill('443');
  await expect(addButton).toBeEnabled();

  await inputs.nth(2).fill(process.env.SHADOWSOCKS_SERVER_PASSWORD!);

  await page.getByTestId('ciphers').click();
  await page
    .getByRole('option', { name: process.env.SHADOWSOCKS_SERVER_CIPHER!, exact: true })
    .click();

  await addButton.click();
  await util.waitForRoute(RoutePath.selectLocation);

  const customBridgeButton = page.locator('button:has-text("Custom bridge")');
  await expect(customBridgeButton).toBeEnabled();

  await expect(page.locator('button[aria-label="Edit custom bridge"]')).toBeVisible();
});

test('App should select custom bridge', async () => {
  const customBridgeButton = page.locator('button:has-text("Custom bridge")');
  await expect(customBridgeButton).toHaveCSS('background-color', colors['--color-green']);

  const automaticButton = page.getByText('Automatic');
  await automaticButton.click();
  await page.getByText(/^Entry$/).click();
  await expect(customBridgeButton).not.toHaveCSS('background-color', colors['--color-green']);

  await customBridgeButton.click();
  await page.getByText(/^Entry$/).click();
  await expect(customBridgeButton).toHaveCSS('background-color', colors['--color-green']);
});

test('App should edit custom bridge', async () => {
  const automaticButton = page.getByText('Automatic');
  await automaticButton.click();
  await page.getByText(/^Entry$/).click();

  await page.click('button[aria-label="Edit custom bridge"]');
  await util.waitForRoute(RoutePath.editCustomBridge);

  const title = page.locator('h1');
  await expect(title).toHaveText('Edit custom bridge');

  const inputs = page.locator('input');
  const saveButton = page.locator('button:has-text("Save")');
  await expect(saveButton).toBeVisible();
  await expect(saveButton).toBeEnabled();

  await inputs.nth(1).fill(process.env.SHADOWSOCKS_SERVER_PORT!);
  await expect(saveButton).toBeEnabled();

  await saveButton.click();
  await util.waitForRoute(RoutePath.selectLocation);

  const customBridgeButton = page.locator('button:has-text("Custom bridge")');
  await expect(customBridgeButton).toBeEnabled();
  await expect(customBridgeButton).toHaveCSS('background-color', colors['--color-green']);
});

test('App should delete custom bridge', async () => {
  await page.click('button[aria-label="Edit custom bridge"]');
  await util.waitForRoute(RoutePath.editCustomBridge);

  const deleteButton = page.locator('button:has-text("Delete")');
  await expect(deleteButton).toBeVisible();
  await expect(deleteButton).toBeEnabled();

  await deleteButton.click();
  await expect(page.getByText('Delete custom bridge?')).toBeVisible();

  const confirmButton = page.getByTestId('delete-confirm');
  await confirmButton.click();
  await util.waitForRoute(RoutePath.selectLocation);

  const customBridgeButton = page.locator('button:has-text("Custom bridge")');
  await expect(customBridgeButton).toBeDisabled();
  await expect(customBridgeButton).not.toHaveCSS('background-color', colors['--color-green']);
});
