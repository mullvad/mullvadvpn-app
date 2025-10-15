import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { RoutePath } from '../../../../src/shared/routes';
import { RoutesObjectModel } from '../../route-object-models';
import { TestUtils } from '../../utils';
import { startInstalledApp } from '../installed-utils';

// This test expects the daemon to be logged out and the provided account to have five registered
// devices..
// Env parameters:
//   `ACCOUNT_NUMBER`: Account number to use when logging in

let page: Page;
let util: TestUtils;
let routes: RoutesObjectModel;

test.beforeAll(async () => {
  ({ page, util } = await startInstalledApp());
  routes = new RoutesObjectModel(page, util);
  await util.expectRoute(RoutePath.login);
});

test.afterAll(async () => {
  await util?.closePage();
});

test('App should show too many devices', async () => {
  const loginInput = routes.login.selectors.loginInput();
  await loginInput.fill(process.env.ACCOUNT_NUMBER!);

  await loginInput.press('Enter');
  await util.expectRoute(RoutePath.tooManyDevices);

  const loginButton = routes.tooManyDevices.selectors.continueButton();
  await expect(loginButton).toBeDisabled();

  const removeDeviceButton = routes.tooManyDevices.selectors.removeDeviceButtons();
  await removeDeviceButton.first().click();

  await routes.tooManyDevices.selectors.confirmRemoveDeviceButton().click();

  await expect(loginButton).toBeEnabled();

  // Trigger transition: too-many-devices -> login -> main
  await loginButton.click();
  await util.expectRoute(RoutePath.login);
  await util.expectRoute(RoutePath.main);
});
