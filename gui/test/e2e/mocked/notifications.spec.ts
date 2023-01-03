import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { colors } from '../../../src/config.json';
import { IAccountData } from '../../../src/shared/daemon-rpc-types';
import { getBackgroundColor, GetByTestId } from '../utils';
import { SendMockIpcResponse, startAppWithMocking } from './mocked-utils';

let page: Page;
let sendMockIpcResponse: SendMockIpcResponse;
let getByTestId: GetByTestId;

test.beforeAll(async () => {
  ({ page, sendMockIpcResponse, getByTestId } = await startAppWithMocking());
});

test.afterAll(async () => {
  await page.close();
});

/**
 * Expires soon
 */
test('App should notify user about account expiring soon', async () => {
  await sendMockIpcResponse<IAccountData>({
    channel: 'account-',
    response: { expiry: new Date(Date.now() + 2 * 24 * 60 * 60 * 1000).toISOString() },
  });

  const title = getByTestId('notificationTitle');
  await expect(title).toContainText(/account credit expires soon/i);

  let subTitle = getByTestId('notificationSubTitle');
  await expect(subTitle).toContainText(/1 day left\. buy more credit\./i);

  const indicator = getByTestId('notificationIndicator');
  const indicatorColor = await getBackgroundColor(indicator);
  expect(indicatorColor).toBe(colors.yellow);

  await sendMockIpcResponse<IAccountData>({
    channel: 'account-',
    response: { expiry: new Date(Date.now() + 3 * 24 * 60 * 60 * 1000).toISOString() },
  });
  subTitle = getByTestId('notificationSubTitle');
  await expect(subTitle).toContainText(/2 days left\. buy more credit\./i);

  await sendMockIpcResponse<IAccountData>({
    channel: 'account-',
    response: { expiry: new Date(Date.now() + 1 * 24 * 60 * 60 * 1000).toISOString() },
  });
  subTitle = getByTestId('notificationSubTitle');
  await expect(subTitle).toContainText(/less than a day left\. buy more credit\./i);
});
