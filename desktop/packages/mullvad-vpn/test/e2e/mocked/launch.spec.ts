import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { RoutesObjectModel } from '../route-object-models';
import { MockedTestUtils, startMockedApp } from './mocked-utils';

let page: Page;
let util: MockedTestUtils;
let routes: RoutesObjectModel;

test.describe('Launch', () => {
  test.beforeAll(async () => {
    ({ page, util } = await startMockedApp());
    routes = new RoutesObjectModel(page, util);
    await routes.main.waitForRoute();

    await util.ipc.daemon.disconnected.notify();
    await routes.launch.waitForRoute();
  });

  test.afterAll(async () => {
    await util?.closePage();
  });

  test.describe('Linux', () => {
    test.skip(() => process.platform !== 'linux');
    test('Should display default footer', async () => {
      const learnMoreButton = routes.launch.selectors.learnMoreButton();
      await expect(learnMoreButton).toBeVisible();
      const defaultFooterText = routes.launch.selectors.defaultFooterText();
      await expect(defaultFooterText).toBeVisible();
    });
  });

  test.describe('Windows', () => {
    test.skip(() => process.platform !== 'win32');
    test('Should display restart daemon footer', async () => {
      const learnMoreButton = routes.launch.selectors.detailsButton();
      await expect(learnMoreButton).toBeVisible();
      const tryAgainButton = routes.launch.selectors.tryAgainButton();
      await expect(tryAgainButton).toBeVisible();
    });
  });
  test.describe('MacOS', () => {
    test.skip(() => process.platform !== 'darwin');
    test('Should display default footer', async () => {
      const learnMoreButton = routes.launch.selectors.learnMoreButton();
      await expect(learnMoreButton).toBeVisible();
      const defaultFooterText = routes.launch.selectors.defaultFooterText();
      await expect(defaultFooterText).toBeVisible();
    });
    test('Should display permission footer when daemon is not allowed', async () => {
      await util.ipc.daemon.daemonAllowed.notify(false);
      const gotoSystemSettingsButton = routes.launch.selectors.gotoSystemSettingsButton();
      await expect(gotoSystemSettingsButton).toBeVisible();
    });
  });

  test('Should navigate to main after establishing connection to daemon', async () => {
    await util.ipc.daemon.connected.notify();
    await routes.main.waitForRoute();
  });
});
