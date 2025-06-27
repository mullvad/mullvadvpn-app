import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { RoutePath } from '../../../../src/shared/routes';
import { RoutesObjectModel } from '../../route-object-models';
import { MockedTestUtils, startMockedApp } from '../mocked-utils';
import { createIpc } from './ipc';

let page: Page;
let util: MockedTestUtils;
let routes: RoutesObjectModel;
let ipc: ReturnType<typeof createIpc>;

test.describe('Launch', () => {
  test.beforeAll(async () => {
    ({ page, util } = await startMockedApp());
    ipc = createIpc(util);
    routes = new RoutesObjectModel(page, util);
    await util.waitForRoute(RoutePath.main);

    await ipc.send.daemonDisconnected();
    await routes.launch.waitForRoute();
  });

  test.afterAll(async () => {
    await page.close();
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
      await ipc.send.daemonAllowed(false);
      const gotoSystemSettingsButton = routes.launch.selectors.gotoSystemSettingsButton();
      await expect(gotoSystemSettingsButton).toBeVisible();
    });
  });
});
