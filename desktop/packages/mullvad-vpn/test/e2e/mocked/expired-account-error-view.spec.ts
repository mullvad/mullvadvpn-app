import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { colorTokens } from '../../../src/renderer/lib/foundations';
import { IAccountData } from '../../../src/shared/daemon-rpc-types';
import { RoutePath } from '../../../src/shared/routes';
import { getBackgroundColor } from '../utils';
import { MockedTestUtils, startMockedApp } from './mocked-utils';

let page: Page;
let util: MockedTestUtils;

test.beforeEach(async () => {
  ({ page, util } = await startMockedApp());
  await util.waitForRoute(RoutePath.main);
});

test.afterEach(async () => {
  await page.close();
});

test('App should show Expired Account Error View', async () => {
  await util.sendMockIpcResponse<IAccountData>({
    channel: 'account-',
    response: { expiry: new Date(Date.now() - 10 * 24 * 60 * 60 * 1000).toISOString() },
  });

  await expect(page.locator('text=Out of time')).toBeVisible();
  const buyMoreButton = page.locator('button:has-text("Buy more credit")');
  await expect(buyMoreButton).toBeVisible();
  expect(await getBackgroundColor(buyMoreButton)).toBe(colorTokens.green);

  const redeemVoucherButton = page.locator('button:has-text("Redeem voucher")');
  await expect(redeemVoucherButton).toBeVisible();
  expect(await getBackgroundColor(redeemVoucherButton)).toBe(colorTokens.green);
});

test('App should show out of time view after running out of time', async () => {
  const expiryDate = new Date();
  expiryDate.setSeconds(expiryDate.getSeconds() + 2);

  await util.sendMockIpcResponse<IAccountData>({
    channel: 'account-',
    response: { expiry: expiryDate.toISOString() },
  });
  await util.waitForRoute(RoutePath.expired);
});
