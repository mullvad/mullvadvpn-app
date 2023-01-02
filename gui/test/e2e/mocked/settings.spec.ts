import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { MockedTestUtils, startMockedApp } from './mocked-utils';
import { IAccountData } from '../../../src/shared/daemon-rpc-types';

let page: Page;
let util: MockedTestUtils;

test.beforeAll(async () => {
  ({ page, util } = await startMockedApp());
  await util.waitForNavigation(() => page.click('button[aria-label="Settings"]'));
});

test.afterAll(async () => {
  await page.close();
});

test('Settings Page', async () => {
  const title = page.locator('h1');
  await expect(title).toContainText('Settings');

  const closeButton = page.locator('button[aria-label="Close"]');
  await expect(closeButton).toBeVisible();
});

test('Account button should be displayed correctly', async () => {
  const accountButton = page.locator('button:has-text("Account")');
  await expect(accountButton).toBeVisible();

  let expiryText = accountButton.locator('span');
  await expect(expiryText).toContainText(/29 days left/i);

  /**
   * 729 days left
   * Add a one-second margin to the test, since it randomly fails in Github Actions otherwise
   */
  await util.sendMockIpcResponse<IAccountData>({
    channel: 'account-',
    response: { expiry: new Date(Date.now() + 730 * 24 * 60 * 60 * 1000 - 1000).toISOString() },
  });
  expiryText = accountButton.locator('span');
  await expect(expiryText).toContainText(/729 days left/i);

  /**
   * 2 years left
   */
  await util.sendMockIpcResponse<IAccountData>({
    channel: 'account-',
    response: { expiry: new Date(Date.now() + 731 * 24 * 60 * 60 * 1000).toISOString() },
  });
  expiryText = accountButton.locator('span');
  await expect(expiryText).toContainText(/2 years left/i);

  /**
   * Expiry 1 day ago should show 'out of time'
   */
  await util.sendMockIpcResponse<IAccountData>({
    channel: 'account-',
    response: { expiry: new Date(Date.now() - 1 * 24 * 60 * 60 * 1000).toISOString() },
  });
  expiryText = accountButton.locator('span');
  await expect(expiryText).toContainText(/out of time/i);
});
