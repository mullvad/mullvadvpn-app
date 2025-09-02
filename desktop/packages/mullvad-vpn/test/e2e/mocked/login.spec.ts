import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { RoutesObjectModel } from '../route-object-models';
import { MockedTestUtils, startMockedApp } from './mocked-utils';

let page: Page;
let util: MockedTestUtils;
let routes: RoutesObjectModel;

test.describe.configure({ mode: 'parallel' });

test.describe('Clear account history warnings', () => {
  const startup = async () => {
    ({ page, util } = await startMockedApp());
    routes = new RoutesObjectModel(page, util);
  };

  const logout = async () => {
    await util.ipc.account.device.notify({
      type: 'logged out',
      deviceState: { type: 'logged out' },
    });

    await routes.login.waitForRoute();
  };

  test.beforeAll(async () => {
    await startup();
    await routes.main.waitForRoute();
  });

  test.beforeEach(async () => {
    await logout();
  });

  test.afterAll(async () => {
    await page.close();
  });

  const setAccountHistory = async () => {
    await util.ipc.accountHistory[''].notify('1234123412341234');
  };

  test('Should not warn about creating an account', async () => {
    const accountHistoryItemButton = routes.login.getAccountHistoryItemButton();
    await expect(accountHistoryItemButton).not.toBeVisible();

    await Promise.all([util.ipc.account.create.expect(), routes.login.createNewAccount()]);
  });

  test('Should warn about creating an account', async () => {
    await setAccountHistory();

    const confirmationMessage = routes.login.getCreateNewAccountConfirmationMessage();
    await expect(confirmationMessage).not.toBeVisible();
    await routes.login.createNewAccount();
    await expect(confirmationMessage).toBeVisible();
    await routes.login.cancelCreateNewAccount();
    await expect(confirmationMessage).not.toBeVisible();

    await routes.login.createNewAccount();

    await Promise.all([util.ipc.account.create.expect(), routes.login.confirmCreateNewAccount()]);
  });

  test('Should warn about clearing account history', async () => {
    await setAccountHistory();

    const accountHistoryItemButton = routes.login.getAccountHistoryItemButton();
    await expect(accountHistoryItemButton).toBeVisible();

    const confirmationMessage = routes.login.getClearAccountHistoryConfirmationMessage();
    await expect(confirmationMessage).not.toBeVisible();
    await routes.login.clearAccountHistory();
    await expect(confirmationMessage).toBeVisible();
    await routes.login.cancelClearAccountHistory();
    await expect(confirmationMessage).not.toBeVisible();
    await expect(accountHistoryItemButton).toBeVisible();

    await routes.login.clearAccountHistory();
    await Promise.all([
      util.ipc.accountHistory.clear.expect(),
      routes.login.confirmClearAccountHistory(),
    ]);

    await util.ipc.accountHistory[''].notify(undefined);
    await expect(accountHistoryItemButton).not.toBeVisible();
  });
});
