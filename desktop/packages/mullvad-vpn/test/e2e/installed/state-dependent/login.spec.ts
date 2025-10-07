import { expect, test } from '@playwright/test';
import { exec, execSync } from 'child_process';
import { Page } from 'playwright';

import { RoutePath } from '../../../../src/shared/routes';
import { RoutesObjectModel } from '../../route-object-models';
import { expectDisconnected } from '../../shared/tunnel-state';
import { TestUtils } from '../../utils';
import { startInstalledApp } from '../installed-utils';

// This test expects the daemon to be logged out and the account history to be cleared.
// Env parameters:
//   `ACCOUNT_NUMBER`: Account number to use when logging in

let page: Page;
let util: TestUtils;
let routes: RoutesObjectModel;

let accountNumber: string;

test.beforeAll(async () => {
  ({ page, util } = await startInstalledApp());
  routes = new RoutesObjectModel(page, util);
});

test.afterAll(async () => {
  await util?.closePage();
});

test('App should fail to login', async () => {
  await util.expectRoute(RoutePath.login);

  const title = page.locator('h1');
  const subtitle = page.getByTestId('subtitle');

  await expect(title).toHaveText('Login');
  await expect(subtitle).toHaveText('Enter your account number');

  await routes.login.fillAccountNumber('1234 1234 1324 1234');
  await routes.login.loginByPressingEnter();

  await expect(title).toHaveText('Login failed');
  await expect(subtitle).toHaveText('Invalid account number');

  await routes.login.fillAccountNumber('');
});

test('App should create account', async () => {
  await util.expectRoute(RoutePath.login);

  await routes.login.createNewAccount();
  await util.expectRoute(RoutePath.expired);

  const outOfTimeTitle = page.getByTestId('title');
  await expect(outOfTimeTitle).toHaveText('Congrats!');

  const inputValue = await page.getByTestId('account-number').textContent();
  expect(inputValue).toHaveLength(19);
  accountNumber = inputValue!.replaceAll(' ', '');
});

test('App should become logged out', async () => {
  exec('mullvad account logout');
  await util.expectRoute(RoutePath.login);
});

test('App should log in', async () => {
  await util.expectRoute(RoutePath.login);

  const title = page.locator('h1');
  const subtitle = page.getByTestId('subtitle');

  await expect(title).toHaveText('Login');
  await expect(subtitle).toHaveText('Enter your account number');

  await routes.login.fillAccountNumber(process.env.ACCOUNT_NUMBER!);
  await routes.login.loginByClickingLoginButton();

  await expect(title).toHaveText('Logged in');
  await expect(subtitle).toHaveText('Valid account number');

  await util.expectRoute(RoutePath.main);

  await expectDisconnected(page);
});

test('App should log out', async () => {
  await page.getByTestId('account-button').click();

  await util.expectRoute(RoutePath.account);

  await page.getByText('Log out').click();
  await util.expectRoute(RoutePath.login);

  const title = page.locator('h1');
  const subtitle = page.getByTestId('subtitle');
  await expect(title).toHaveText('Login');
  await expect(subtitle).toHaveText('Enter your account number');
});

test('App should log in to expired account', async () => {
  await util.expectRoute(RoutePath.login);

  const title = page.locator('h1');
  const subtitle = page.getByTestId('subtitle');

  await expect(title).toHaveText('Login');
  await expect(subtitle).toHaveText('Enter your account number');

  await routes.login.fillAccountNumber(accountNumber);

  await routes.login.loginByPressingEnter();
  await util.expectRoute(RoutePath.expired);

  const outOfTimeTitle = page.getByTestId('title');
  await expect(outOfTimeTitle).toHaveText('Out of time');

  execSync('mullvad account logout');
});
