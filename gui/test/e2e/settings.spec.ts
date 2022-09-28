import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { sendMockIpcResponse, startApp } from './utils';
import { IAccountData } from '../../src/shared/daemon-rpc-types';

let appWindow: Page;

test.beforeAll(async () => {
  const startAppResponse = await startApp();
  appWindow = startAppResponse.appWindow;
  await appWindow.click('button[aria-label="Settings"]');
});

test.afterAll(async () => {
  await appWindow.close();
});

test('Settings Page', async () => {
  const title = appWindow.locator('h1');
  await expect(title).toContainText('Settings');

  const closeButton = appWindow.locator('button[aria-label="Close"]');
  await expect(closeButton).toBeVisible();
});

test('Account button should be displayed correctly', async () => {
  const accountButton = appWindow.locator('button:has-text("Account")');
  await expect(accountButton).toBeVisible();

  let expiryText = accountButton.locator('span');
  await expect(expiryText).toContainText(/29 days left/i);

  /**
   * 729 days left
   */
  await sendMockIpcResponse<IAccountData>({
    channel: 'account-',
    response: { expiry: new Date(Date.now() + 730 * 24 * 60 * 60 * 1000).toISOString() },
  });
  expiryText = accountButton.locator('span');
  await expect(expiryText).toContainText(/729 days left/i);

  /**
   * 2 years left
   */
  await sendMockIpcResponse<IAccountData>({
    channel: 'account-',
    response: { expiry: new Date(Date.now() + 731 * 24 * 60 * 60 * 1000).toISOString() },
  });
  expiryText = accountButton.locator('span');
  await expect(expiryText).toContainText(/2 years left/i);

  /**
   * Expiry 1 day ago should show 'out of time'
   */
  await sendMockIpcResponse<IAccountData>({
    channel: 'account-',
    response: { expiry: new Date(Date.now() - 1 * 24 * 60 * 60 * 1000).toISOString() },
  });
  expiryText = accountButton.locator('span');
  await expect(expiryText).toContainText(/out of time/i);
});
