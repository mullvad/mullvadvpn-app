import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { colors } from '../../../src/renderer/lib/foundations';
import { RoutePath } from '../../../src/renderer/lib/routes';
import { IAccountData } from '../../../src/shared/daemon-rpc-types';
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
  expect(await getBackgroundColor(buyMoreButton)).toBe(colors['--color-green']);

  const redeemVoucherButton = page.locator('button:has-text("Redeem voucher")');
  await expect(redeemVoucherButton).toBeVisible();
  expect(await getBackgroundColor(redeemVoucherButton)).toBe(colors['--color-green']);
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
