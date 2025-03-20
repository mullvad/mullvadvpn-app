import { expect, test } from '@playwright/test';
import { Locator, Page } from 'playwright';

import { RoutePath } from '../../../../src/renderer/lib/routes';
import { TestUtils } from '../../utils';
import { startInstalledApp } from '../installed-utils';

// This test expects the daemon to be logged out and the provided account to have five registered
// devices..
// Env parameters:
//   `ACCOUNT_NUMBER`: Account number to use when logging in

let page: Page;
let util: TestUtils;

test.beforeAll(async () => {
  ({ page, util } = await startInstalledApp());
  await util.waitForRoute(RoutePath.login);
});

test.afterAll(async () => {
  await page.close();
});

test('App should show too many devices', async () => {
  const loginInput = getInput(page);
  await loginInput.fill(process.env.ACCOUNT_NUMBER!);

  await loginInput.press('Enter');
  await util.waitForRoute(RoutePath.tooManyDevices);

  const loginButton = page.getByText('Continue with login');

  await expect(page.getByTestId('title')).toHaveText('Too many devices');
  await expect(loginButton).toBeDisabled();
  await page
    .getByLabel(/^Remove device named/)
    .first()
    .click();
  await page.getByText('Yes, log out device').click();

  await expect(loginButton).toBeEnabled();

  // Trigger transition: too-many-devices -> login -> main
  await loginButton.click();
  await util.waitForRoute(RoutePath.login);
  await util.waitForRoute(RoutePath.main);
  await expect(page.getByTestId(RoutePath.main)).toBeVisible();
});

function getInput(page: Page): Locator {
  return page.getByPlaceholder('0000 0000 0000 0000');
}
