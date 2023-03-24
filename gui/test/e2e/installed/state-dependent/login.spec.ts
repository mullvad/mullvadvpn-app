import { exec } from 'child_process';
import { expect, test } from '@playwright/test';
import { Locator, Page } from 'playwright';
import { RoutePath } from '../../../../src/renderer/lib/routes';
import { TestUtils } from '../../utils';

import { startInstalledApp } from '../installed-utils';

// This test expects the daemon to be logged out.

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

  await expect(title).toHaveText('Logging in...');
  await expect(subtitle).toHaveText('Checking account number');
  await expect(title).toHaveText('Login failed');
  await expect(subtitle).toHaveText('Invalid account number');

  loginInput.fill('');
});

test('App should create account', async () => {
  expect(await util.currentRoute()).toEqual(RoutePath.login);

  const title = page.locator('h1')
  const subtitle = page.getByTestId('subtitle');

  await page.getByText('Create account').click();
  await expect(title).toHaveText('Creating account...');
  await expect(subtitle).toHaveText('Please wait');

  await expect(title).toHaveText('Account created');
  await expect(subtitle).toHaveText('Logged in');

  expect(await util.waitForNavigation()).toEqual(RoutePath.main);

  const inputValue = await page.getByTestId('account-number').textContent();
  expect(inputValue).toHaveLength(19);
  accountNumber = inputValue!.replaceAll(' ', '');
});

test('App should log out', async () => {
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

  await loginInput.type(accountNumber);
  await loginInput.press('Enter');

  await expect(title).toHaveText('Logging in...');
  await expect(subtitle).toHaveText('Checking account number');
  await expect(title).toHaveText('Logged in');
  await expect(subtitle).toHaveText('Valid account number');

  expect(await util.waitForNavigation()).toEqual(RoutePath.main);
});

function getInput(page: Page): Locator {
  return page.getByPlaceholder('0000 0000 0000 0000');
}
