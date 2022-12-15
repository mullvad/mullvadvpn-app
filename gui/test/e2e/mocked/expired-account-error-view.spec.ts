import { Page } from 'playwright';
import { MockedTestUtils, startMockedApp } from './mocked-utils';
import { expect, test } from '@playwright/test';
import { IAccountData } from '../../../src/shared/daemon-rpc-types';
import { getBackgroundColor } from '../utils';
import { colors } from '../../../src/config.json';

let page: Page;
let util: MockedTestUtils;

test.beforeAll(async () => {
  ({ page, util } = await startMockedApp());
});

test.afterAll(async () => {
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
  expect(await getBackgroundColor(buyMoreButton)).toBe(colors.green);

  const redeemVoucherButton = page.locator('button:has-text("Redeem voucher")');
  await expect(redeemVoucherButton).toBeVisible();
  expect(await getBackgroundColor(redeemVoucherButton)).toBe(colors.green);
});
