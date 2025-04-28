import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { RoutePath } from '../../../src/renderer/lib/routes';
import { IAccountData } from '../../../src/shared/daemon-rpc-types';
import { MockedTestUtils, startMockedApp } from './mocked-utils';

let page: Page;
let util: MockedTestUtils;

test.beforeAll(async () => {
  ({ page, util } = await startMockedApp());
  await util.waitForRoute(RoutePath.main);
});

test.afterAll(async () => {
  await page.close();
});

test('Account button should be displayed correctly', async () => {
  const accountButton = page.getByLabel('Account settings');
  await expect(accountButton).toBeVisible();
});

test('Headerbar account info should be displayed correctly', async () => {
  const expiryText = page.getByText(/^Time left:/);
  await expect(expiryText).toContainText(/Time left: 29 days/i);

  /**
   * 729 days left
   * Add a one-second margin to the test, since it randomly fails in Github Actions otherwise
   */
  await util.sendMockIpcResponse<IAccountData>({
    channel: 'account-',
    response: { expiry: new Date(Date.now() + 730 * 24 * 60 * 60 * 1000 - 1000).toISOString() },
  });
  await expect(expiryText).toContainText(/Time left: 729 days/i);

  /**
   * 2 years left
   */
  await util.sendMockIpcResponse<IAccountData>({
    channel: 'account-',
    response: { expiry: new Date(Date.now() + 731 * 24 * 60 * 60 * 1000).toISOString() },
  });
  await expect(expiryText).toContainText(/Time left: 2 years/i);

  /**
   * Expiry 1 day ago should show 'out of time'
   */
  await util.sendMockIpcResponse<IAccountData>({
    channel: 'account-',
    response: { expiry: new Date(Date.now() - 1 * 24 * 60 * 60 * 1000).toISOString() },
  });
  await expect(expiryText).not.toBeVisible();
});

test('Settings Page', async () => {
  await page.click('button[aria-label="Settings"]');
  await util.waitForRoute(RoutePath.settings);

  const title = page.locator('h1');
  await expect(title).toContainText('Settings');

  const closeButton = page.locator('button[aria-label="Close"]');
  await expect(closeButton).toBeVisible();
});
