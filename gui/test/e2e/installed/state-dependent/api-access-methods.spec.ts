import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { startInstalledApp } from '../installed-utils';
import { TestUtils } from '../../utils';
import { RoutePath } from '../../../../src/renderer/lib/routes';

// This test expects the daemon to be logged in and only have "Direct" and "Mullvad Bridges"
// access methods.
// Env parameters:
//   `SHADOWSOCKS_SERVER_IP`: Account number to use when logging in

const DIRECT_NAME = 'Direct';
const BRIDGES_NAME = 'Mullvad Bridges';
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
  await util.waitForNavigation(async () => await page.click('button[aria-label="Settings"]'));
  await util.waitForNavigation(async () => await page.getByText('API access').click());

  const title = page.locator('h1')
  await expect(title).toHaveText('API access');
}

test('App should display access methods', async () => {
  await navigateToAccessMethods();

  const accessMethods = page.getByTestId('access-method');
  await expect(accessMethods).toHaveCount(2);

  const direct = accessMethods.first();
  const bridges = accessMethods.last();
  await expect(direct).toContainText(DIRECT_NAME);
  await expect(direct).toContainText(IN_USE_LABEL);
  await expect(bridges).toHaveText(BRIDGES_NAME);
  await expect(bridges).not.toContainText(IN_USE_LABEL);
});

test('App should add access method', async () => {
  await util.waitForNavigation(async () => await page.locator('button:has-text("Add")').click());

  const title = page.locator('h1')
  await expect(title).toHaveText('Add method');

  const inputs = page.locator('input');
  const addButton = page.locator('button:has-text("Add")');
  expect(addButton).toBeDisabled();

  await inputs.first().fill(FUNCTIONING_METHOD_NAME);
  expect(addButton).toBeDisabled();

  await inputs.nth(1).fill(process.env.SHADOWSOCKS_SERVER_IP!);
  expect(addButton).toBeDisabled();

  await inputs.nth(2).fill('443');
  expect(addButton).toBeEnabled();

  await inputs.nth(3).fill('mullvad');

  await page.getByTestId('ciphers').click();
  await page.getByRole('option', { name: 'aes-256-gcm' }).click();

  expect(
    await util.waitForNavigation(async () => await addButton.click())
  ).toEqual(RoutePath.apiAccessMethods);

  const accessMethods = page.getByTestId('access-method');
  await expect(accessMethods).toHaveCount(3);

  await expect(accessMethods.last()).toHaveText(FUNCTIONING_METHOD_NAME);
});

test('App should add invalid access method', async () => {
  await util.waitForNavigation(async () => await page.locator('button:has-text("Add")').click());

  const title = page.locator('h1')
  await expect(title).toHaveText('Add method');

  const inputs = page.locator('input');
  const addButton = page.locator('button:has-text("Add")');
  expect(addButton).toBeDisabled();

  await inputs.first().fill(NON_FUNCTIONING_METHOD_NAME);
  expect(addButton).toBeDisabled();

  await inputs.nth(1).fill(process.env.SHADOWSOCKS_SERVER_IP!);
  expect(addButton).toBeDisabled();

  await inputs.nth(2).fill('443');
  expect(addButton).toBeEnabled();

  await addButton.click()

  await expect(page.getByText('Testing method...')).toBeVisible();
  await expect(page.getByText('API unreachable, add anyway?')).toBeVisible();

  expect(
    await util.waitForNavigation(async () => await page.locator('button:has-text("Save")').click())
  ).toEqual(RoutePath.apiAccessMethods);

  const accessMethods = page.getByTestId('access-method');
  await expect(accessMethods).toHaveCount(4);

  await expect(accessMethods.last()).toHaveText(NON_FUNCTIONING_METHOD_NAME);
});

test('App should test and use methods', async () => {
  const accessMethods = page.getByTestId('access-method');

  const direct = accessMethods.first();
  const bridges = accessMethods.nth(1);
  const functioningTestMethod = accessMethods.nth(2);
  const nonFunctioningTestMethod = accessMethods.last();

  await expect(direct).toContainText(DIRECT_NAME);
  await expect(direct).toContainText(IN_USE_LABEL);
  await expect(bridges).toHaveText(BRIDGES_NAME);
  await expect(functioningTestMethod).toHaveText(FUNCTIONING_METHOD_NAME);

  await functioningTestMethod.locator('button').last().click();
  await functioningTestMethod.getByText('Use').click();
  await expect(direct).not.toContainText(IN_USE_LABEL);
  await expect(functioningTestMethod).toContainText('API reachable');
  await expect(functioningTestMethod).toContainText(IN_USE_LABEL);

  await nonFunctioningTestMethod.locator('button').last().click();
  await nonFunctioningTestMethod.getByText('Use').click();
  await expect(nonFunctioningTestMethod).toContainText('Testing...');
  await expect(nonFunctioningTestMethod).toContainText('API unreachable');
  await expect(functioningTestMethod).toContainText(IN_USE_LABEL);
});

test('App should delete method', async () => {
  const accessMethods = page.getByTestId('access-method');
  const functioningTestMethod = accessMethods.nth(2);
  const nonFunctioningTestMethod = accessMethods.last();

  await nonFunctioningTestMethod.locator('button').last().click();
  await nonFunctioningTestMethod.getByText('Delete').click();

  await expect(page.getByText(`Delete ${NON_FUNCTIONING_METHOD_NAME}?`)).toBeVisible();
  await page.locator('button:has-text("Delete")').click();
  await expect(accessMethods).toHaveCount(3);

  await functioningTestMethod.locator('button').last().click();
  await functioningTestMethod.getByText('Delete').click();

  await expect(page.getByText(`Delete ${FUNCTIONING_METHOD_NAME}?`)).toBeVisible();
  await page.locator('button:has-text("Delete")').click();
  await expect(accessMethods).toHaveCount(2);
});
