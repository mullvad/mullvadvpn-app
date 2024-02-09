import { exec, execSync } from 'child_process';
import { expect, test } from '@playwright/test';
import { Locator, Page } from 'playwright';
import { RoutePath } from '../../../../src/renderer/lib/routes';
import { TestUtils } from '../../utils';

import { startInstalledApp } from '../installed-utils';
import { expectDisconnected } from '../../shared/tunnel-state';

// This test expects the daemon to be logged out.
// Env parameters:
//   `ACCOUNT_NUMBER`: Account number to use when logging in

let page: Page;
let util: TestUtils;

let accountNumber: string;

test.beforeAll(async () => {
  ({ page, util } = await startInstalledApp());
});

test.afterAll(async () => {
  await page.close();
});

test('App should fail to login', async () => {
  expect(await util.currentRoute()).toEqual(RoutePath.login);

  const title = page.locator('h1')
  const subtitle = page.getByTestId('subtitle');
  const loginInput = getInput(page);

  await expect(title).toHaveText('Login');
  await expect(subtitle).toHaveText('Enter your account number');

  await loginInput.fill('1234 1234 1324 1234');
  await loginInput.press('Enter');

  await expect(title).toHaveText('Login failed');
  await expect(subtitle).toHaveText('Invalid account number');

  await loginInput.fill('');
});

test('App should create account', async () => {
  expect(await util.currentRoute()).toEqual(RoutePath.login);

  const title = page.locator('h1')
  const subtitle = page.getByTestId('subtitle');

  expect(await util.waitForNavigation(async () => {
    await page.getByText('Create account').click();

    await expect(title).toHaveText('Account created');
    await expect(subtitle).toHaveText('Logged in');
  })).toEqual(RoutePath.expired);

  const outOfTimeTitle = page.getByTestId('title');
  await expect(outOfTimeTitle).toHaveText('Congrats!');

  const inputValue = await page.getByTestId('account-number').textContent();
  expect(inputValue).toHaveLength(19);
  accountNumber = inputValue!.replaceAll(' ', '');
});

test('App should become logged out', async () => {
  expect(await util.waitForNavigation(() => {
    exec('mullvad account logout');
  })).toEqual(RoutePath.login);
});

test('App should log in', async () => {
  expect(await util.currentRoute()).toEqual(RoutePath.login);

  const title = page.locator('h1')
  const subtitle = page.getByTestId('subtitle');
  const loginInput = getInput(page);

  await expect(title).toHaveText('Login');
  await expect(subtitle).toHaveText('Enter your account number');

  await loginInput.fill(process.env.ACCOUNT_NUMBER!);

  expect(await util.waitForNavigation(async () => {
    await loginInput.press('Enter');

    await expect(title).toHaveText('Logged in');
    await expect(subtitle).toHaveText('Valid account number');
  })).toEqual(RoutePath.main);
  await expectDisconnected(page);
});

test('App should log out', async () => {
  expect(await util.waitForNavigation(() => {
    void page.getByTestId('account-button').click();
  })).toEqual(RoutePath.account);

  expect(await util.waitForNavigation(() => {
    void page.getByText('Log out').click();
  })).toEqual(RoutePath.login);

  const title = page.locator('h1')
  const subtitle = page.getByTestId('subtitle');
  await expect(title).toHaveText('Login');
  await expect(subtitle).toHaveText('Enter your account number');
});

test('App should log in to expired account', async () => {
  expect(await util.currentRoute()).toEqual(RoutePath.login);

  const title = page.locator('h1')
  const subtitle = page.getByTestId('subtitle');
  const loginInput = getInput(page);

  await expect(title).toHaveText('Login');
  await expect(subtitle).toHaveText('Enter your account number');

  await loginInput.fill(accountNumber);

  expect(await util.waitForNavigation(async () => {
    await loginInput.press('Enter');
  })).toEqual(RoutePath.expired);

  const outOfTimeTitle = page.getByTestId('title');
  await expect(outOfTimeTitle).toHaveText('Out of time');

  execSync('mullvad account logout');
});

function getInput(page: Page): Locator {
  return page.getByPlaceholder('0000 0000 0000 0000');
}
