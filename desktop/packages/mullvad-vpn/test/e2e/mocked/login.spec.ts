import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { RoutesObjectModel } from '../route-object-models';
import { MockedTestUtils, startMockedApp } from './mocked-utils';

const START_DATE = new Date('2025-01-01T13:37:00');

const NON_EXPIRED_EXPIRY = {
  expiry: new Date(START_DATE.getTime() + 60 * 60 * 1000).toISOString(),
};

let page: Page;
let util: MockedTestUtils;
let routes: RoutesObjectModel;

test.describe('Login view', () => {
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
    await page.clock.install({ time: START_DATE });
    await logout();
  });

  test.afterAll(async () => {
    await util?.closePage();
  });

  const setAccountHistory = async () => {
    await util.ipc.accountHistory[''].notify('1234123412341234');
  };

  test('Should login when clicking login button', async () => {
    await routes.login.fillAccountNumber('1234 1234 1234 1234');

    await Promise.all([util.ipc.account.login.expect(), routes.login.loginByClickingLoginButton()]);
    const header = routes.login.selectors.header();
    await expect(header).toHaveText('Logging in...');
    await expect(routes.login.selectors.loginButton()).toBeDisabled();

    await util.ipc.account.device.notify({
      type: 'logged in',
      deviceState: { type: 'logged in', accountAndDevice: { accountNumber: '1234123412341234' } },
    });
    await util.ipc.account[''].notify(NON_EXPIRED_EXPIRY);

    await expect(header).toHaveText('Logged in');
    await page.clock.fastForward(1000);
    await routes.main.waitForRoute();
  });

  test('Should try to login when pressing enter', async () => {
    await routes.login.fillAccountNumber('1234 1234 1234 1234');

    await Promise.all([util.ipc.account.login.expect(), routes.login.loginByPressingEnter()]);
    const header = routes.login.selectors.header();
    await expect(header).toHaveText('Logging in...');
    await expect(routes.login.selectors.loginButton()).toBeDisabled();
  });

  test('Should disable login button when input is invalid', async () => {
    const loginButton = routes.login.selectors.loginButton();
    await expect(loginButton).toBeDisabled();

    await routes.login.fillAccountNumber('1234 1234');
    await expect(loginButton).toBeDisabled();
  });

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
