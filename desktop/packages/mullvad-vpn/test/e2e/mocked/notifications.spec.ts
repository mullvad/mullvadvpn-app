import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { colors } from '../../../src/renderer/lib/foundations';
import { RoutePath } from '../../../src/renderer/lib/routes';
import { IAccountData } from '../../../src/shared/daemon-rpc-types';
import { getBackgroundColor } from '../utils';
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

/**
 * Expires soon
 */
test('App should notify user about account expiring soon', async () => {
  await util.sendMockIpcResponse<IAccountData>({
    channel: 'account-',
    response: { expiry: new Date(Date.now() + 2 * 24 * 60 * 60 * 1000).toISOString() },
  });

  const title = page.getByTestId('notificationTitle');
  await expect(title).toContainText(/account credit expires soon/i);

  let subTitle = page.getByTestId('notificationSubTitle');
  await expect(subTitle).toContainText(/1 day left\. buy more credit\./i);

  const indicator = page.getByTestId('notificationIndicator');
  const indicatorColor = await getBackgroundColor(indicator);
  expect(indicatorColor).toBe(colors['--color-yellow']);

  await util.sendMockIpcResponse<IAccountData>({
    channel: 'account-',
    response: { expiry: new Date(Date.now() + 3 * 24 * 60 * 60 * 1000).toISOString() },
  });
  subTitle = page.getByTestId('notificationSubTitle');
  await expect(subTitle).toContainText(/2 days left\. buy more credit\./i);

  await util.sendMockIpcResponse<IAccountData>({
    channel: 'account-',
    response: { expiry: new Date(Date.now() + 1 * 24 * 60 * 60 * 1000).toISOString() },
  });
  subTitle = page.getByTestId('notificationSubTitle');
  await expect(subTitle).toContainText(/less than a day left\. buy more credit\./i);
});
