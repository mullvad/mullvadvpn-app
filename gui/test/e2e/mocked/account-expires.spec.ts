import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { MockedTestUtils, startMockedApp } from './mocked-utils';
import { IAccountData } from '../../../src/shared/daemon-rpc-types';
import { RoutePath } from '../../../src/renderer/lib/routes';

let page: Page;
let util: MockedTestUtils;

test.beforeAll(async () => {
  ({ page, util } = await startMockedApp());
});

test.afterAll(async () => {
  await page.close();
});

test('App should show out of time view after running out of time', async () => {
  const expiryDate = new Date();
  expiryDate.setSeconds(expiryDate.getSeconds() + 3);

  expect(await util.waitForNavigation(async () => {
    await util.sendMockIpcResponse<IAccountData>({
      channel: 'account-',
      response: { expiry: expiryDate.toISOString() },
    });
  })).toEqual(RoutePath.expired);
});
