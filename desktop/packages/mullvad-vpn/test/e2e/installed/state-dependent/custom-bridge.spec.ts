import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { colors } from '../../../../src/config.json';
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
});

test.afterAll(async () => {
  await page.close();
});

test('App should enable bridge mode', async () => {
  await util.waitForNavigation(() => page.click('button[aria-label="Settings"]'));
  expect(await util.waitForNavigation(() => page.getByText('VPN settings').click())).toBe(
    RoutePath.vpnSettings,
  );

  await page.getByRole('option', { name: 'OpenVPN' }).click();

  expect(await util.waitForNavigation(() => page.getByText('OpenVPN settings').click())).toBe(
    RoutePath.openVpnSettings,
  );

  await page.getByTestId('bridge-mode-on').click();
  await expect(page.getByText('Enable bridge mode?')).toBeVisible();

  await page.getByTestId('enable-confirm').click();

  await util.waitForNavigation(() => page.click('button[aria-label="Back"]'));
  await util.waitForNavigation(() => page.click('button[aria-label="Back"]'));
  expect(await util.waitForNavigation(() => page.click('button[aria-label="Close"]'))).toBe(
    RoutePath.main,
  );
});

test('App display disabled custom bridge', async () => {
  expect(
    await util.waitForNavigation(() => page.click('button[aria-label^="Select location"]')),
  ).toBe(RoutePath.selectLocation);

  const title = page.locator('h1');
  await expect(title).toHaveText('Select location');

  await page.getByText(/^Entry$/).click();

  const customBridgeButton = page.getByText('Custom bridge');
  await expect(customBridgeButton).toBeDisabled();
});

test('App should add new custom bridge', async () => {
  expect(
    await util.waitForNavigation(() => page.click('button[aria-label="Add new custom bridge"]')),
  ).toBe(RoutePath.editCustomBridge);

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

  expect(await util.waitForNavigation(() => addButton.click())).toEqual(RoutePath.selectLocation);

  const customBridgeButton = page.getByText('Custom bridge');
  await expect(customBridgeButton).toBeEnabled();

  await expect(page.locator('button[aria-label="Edit custom bridge"]')).toBeVisible();
});

test('App should select custom bridge', async () => {
  const customBridgeButton = page.locator('button:has-text("Custom bridge")');
  await expect(customBridgeButton).toHaveCSS('background-color', colors.green);

  const automaticButton = page.getByText('Automatic');
  await automaticButton.click();
  await page.getByText(/^Entry$/).click();
  await expect(customBridgeButton).not.toHaveCSS('background-color', colors.green);

  await customBridgeButton.click();
  await page.getByText(/^Entry$/).click();
  await expect(customBridgeButton).toHaveCSS('background-color', colors.green);
});

test('App should edit custom bridge', async () => {
  const automaticButton = page.getByText('Automatic');
  await automaticButton.click();
  await page.getByText(/^Entry$/).click();

  expect(
    await util.waitForNavigation(() => page.click('button[aria-label="Edit custom bridge"]')),
  ).toBe(RoutePath.editCustomBridge);

  const title = page.locator('h1');
  await expect(title).toHaveText('Edit custom bridge');

  const inputs = page.locator('input');
  const saveButton = page.locator('button:has-text("Save")');
  await expect(saveButton).toBeVisible();
  await expect(saveButton).toBeEnabled();

  await inputs.nth(1).fill(process.env.SHADOWSOCKS_SERVER_PORT!);
  await expect(saveButton).toBeEnabled();

  expect(await util.waitForNavigation(() => saveButton.click())).toEqual(RoutePath.selectLocation);

  const customBridgeButton = page.locator('button:has-text("Custom bridge")');
  await expect(customBridgeButton).toBeEnabled();
  await expect(customBridgeButton).toHaveCSS('background-color', colors.green);
});

test('App should delete custom bridge', async () => {
  expect(
    await util.waitForNavigation(() => page.click('button[aria-label="Edit custom bridge"]')),
  ).toBe(RoutePath.editCustomBridge);

  const deleteButton = page.locator('button:has-text("Delete")');
  await expect(deleteButton).toBeVisible();
  await expect(deleteButton).toBeEnabled();

  await deleteButton.click();
  await expect(page.getByText('Delete custom bridge?')).toBeVisible();

  const confirmButton = page.getByTestId('delete-confirm');
  expect(await util.waitForNavigation(() => confirmButton.click())).toEqual(
    RoutePath.selectLocation,
  );

  const customBridgeButton = page.locator('button:has-text("Custom bridge")');
  await expect(customBridgeButton).toBeDisabled();
  await expect(customBridgeButton).not.toHaveCSS('background-color', colors.green);
});
