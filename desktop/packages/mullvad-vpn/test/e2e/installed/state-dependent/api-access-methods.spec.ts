import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { RoutePath } from '../../../../src/renderer/lib/routes';
import { TestUtils } from '../../utils';
import { startInstalledApp } from '../installed-utils';

// This test expects the daemon to be logged in and only have "Direct" and "Mullvad Bridges"
// access methods.
// Env parameters:
//   `SHADOWSOCKS_SERVER_IP`
//   `SHADOWSOCKS_SERVER_PORT`
//   `SHADOWSOCKS_SERVER_CIPHER`
//   `SHADOWSOCKS_SERVER_PASSWORD`

const DIRECT_NAME = 'Direct';
const BRIDGES_NAME = 'Mullvad Bridges';
const ENCRYPTED_DNS_PROXY_NAME = 'Encrypted DNS proxy';
const IN_USE_LABEL = 'In use';
const FUNCTIONING_METHOD_NAME = 'Test method';
const NON_FUNCTIONING_METHOD_NAME = 'Non functioning test method';

let page: Page;
let util: TestUtils;

test.beforeAll(async () => {
  ({ page, util } = await startInstalledApp());
});

test.afterAll(async () => {
  await page.close();
});

async function navigateToAccessMethods() {
  await page.click('button[aria-label="Settings"]');
  await util.waitForRoute(RoutePath.settings);
  await page.getByText('API access').click();
  await util.waitForRoute(RoutePath.apiAccessMethods);

  const title = page.locator('h1');
  await expect(title).toHaveText('API access');
}

test('App should display access methods', async () => {
  await navigateToAccessMethods();

  const accessMethods = page.getByTestId('access-method');
  await expect(accessMethods).toHaveCount(3);

  const direct = accessMethods.first();
  const bridges = accessMethods.nth(1);
  const encryptedDnsProxy = accessMethods.nth(2);
  await expect(direct).toContainText(DIRECT_NAME);
  await expect(bridges).toContainText(BRIDGES_NAME);
  await expect(encryptedDnsProxy).toContainText(ENCRYPTED_DNS_PROXY_NAME);
  await expect(page.getByText(IN_USE_LABEL)).toHaveCount(1);
});

test('App should add invalid access method', async () => {
  await page.locator('button:has-text("Add")').click();
  await util.waitForNextRoute();

  const title = page.locator('h1');
  await expect(title).toHaveText('Add method');

  const inputs = page.locator('input');
  const addButton = page.locator('button:has-text("Add")');
  await expect(addButton).toBeVisible();
  await expect(addButton).toBeDisabled();

  await inputs.first().fill(NON_FUNCTIONING_METHOD_NAME);
  await expect(addButton).toBeDisabled();

  await inputs.nth(1).fill(process.env.SHADOWSOCKS_SERVER_IP!);
  await expect(addButton).toBeDisabled();

  await inputs.nth(2).fill(process.env.SHADOWSOCKS_SERVER_PORT!);
  await expect(addButton).toBeEnabled();

  await addButton.click();

  await expect(page.getByText('Testing method...')).toBeVisible();
  await expect(page.getByText('API unreachable, add anyway?')).toBeVisible();

  await page.locator('button:has-text("Save")').click();
  await util.waitForRoute(RoutePath.apiAccessMethods);

  const accessMethods = page.getByTestId('access-method');
  // Direct, Bridges, Encrypted DNS Proxy & the non-functioning access method.
  await expect(accessMethods).toHaveCount(4);

  await expect(accessMethods.last()).toHaveText(NON_FUNCTIONING_METHOD_NAME);
});

test('App should use invalid method', async () => {
  const accessMethods = page.getByTestId('access-method');
  const nonFunctioningTestMethod = accessMethods.last();

  await expect(page.getByText(IN_USE_LABEL)).toHaveCount(1);
  await expect(nonFunctioningTestMethod).not.toContainText(IN_USE_LABEL);

  await nonFunctioningTestMethod.locator('button').last().click();
  await nonFunctioningTestMethod.getByText('Use').click();
  await expect(nonFunctioningTestMethod).toContainText('Testing...');
  await expect(nonFunctioningTestMethod).toContainText('API unreachable');

  await expect(page.getByText(IN_USE_LABEL)).toHaveCount(1);
  await expect(nonFunctioningTestMethod).not.toContainText(IN_USE_LABEL);
});

test('App should edit access method', async () => {
  const customMethod = page.getByTestId('access-method').last();
  await customMethod.locator('button').last().click();
  await customMethod.getByText('Edit').click();
  await util.waitForNextRoute();

  const title = page.locator('h1');
  await expect(title).toHaveText('Edit method');

  const inputs = page.locator('input');
  const saveButton = page.locator('button:has-text("Save")');
  await expect(saveButton).toBeVisible();
  await expect(saveButton).toBeEnabled();

  await expect(inputs.first()).toHaveValue(NON_FUNCTIONING_METHOD_NAME);
  await expect(inputs.nth(1)).toHaveValue(process.env.SHADOWSOCKS_SERVER_IP!);
  await expect(inputs.nth(2)).toHaveValue(process.env.SHADOWSOCKS_SERVER_PORT!);

  await inputs.first().fill(FUNCTIONING_METHOD_NAME);
  await expect(saveButton).toBeEnabled();

  await inputs.nth(3).fill(process.env.SHADOWSOCKS_SERVER_PASSWORD!);

  await page.getByTestId('ciphers').click();
  await page
    .getByRole('option', { name: process.env.SHADOWSOCKS_SERVER_CIPHER!, exact: true })
    .click();

  await page.locator('button:has-text("Save")').last().click();
  await util.waitForRoute(RoutePath.apiAccessMethods);

  const accessMethods = page.getByTestId('access-method');
  // Direct, Bridges, Encrypted DNS Proxy & the custom access method.
  await expect(accessMethods).toHaveCount(4);

  await expect(accessMethods.last()).toHaveText(FUNCTIONING_METHOD_NAME);
});

test('App should use valid method', async () => {
  const accessMethods = page.getByTestId('access-method');

  const direct = accessMethods.first();
  const bridges = accessMethods.nth(1);
  const encryptedDnsProxy = accessMethods.nth(2);
  const functioningTestMethod = accessMethods.last();

  await expect(page.getByText(IN_USE_LABEL)).toHaveCount(1);
  await expect(functioningTestMethod).not.toContainText(IN_USE_LABEL);
  await expect(functioningTestMethod).toHaveText(FUNCTIONING_METHOD_NAME);

  await functioningTestMethod.locator('button').last().click();
  await functioningTestMethod.getByText('Use').click();
  await expect(direct).not.toContainText(IN_USE_LABEL);
  await expect(bridges).not.toContainText(IN_USE_LABEL);
  await expect(encryptedDnsProxy).not.toContainText(IN_USE_LABEL);
  await expect(functioningTestMethod).toContainText('API reachable');
  await expect(functioningTestMethod).toContainText(IN_USE_LABEL);
});

test('App should delete method', async () => {
  const accessMethods = page.getByTestId('access-method');
  const customMethod = accessMethods.last();

  await customMethod.locator('button').last().click();
  await customMethod.getByText('Delete').click();

  await expect(page.getByText(`Delete ${FUNCTIONING_METHOD_NAME}?`)).toBeVisible();
  await page.locator('button:has-text("Delete")').click();
  // Direct, Bridges, Encrypted DNS Proxy.
  await expect(accessMethods).toHaveCount(3);
});
