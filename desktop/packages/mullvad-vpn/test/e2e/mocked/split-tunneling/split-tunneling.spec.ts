import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { RoutePath } from '../../../../src/shared/routes';
import { RoutesObjectModel } from '../../route-object-models';
import { MockedTestUtils, startMockedApp } from '../mocked-utils';
import { linuxApplicationsList } from './helpers';

let page: Page;
let util: MockedTestUtils;
let routes: RoutesObjectModel;

test.describe('Split tunneling', () => {
  test.beforeAll(async () => {
    ({ page, util } = await startMockedApp());
    routes = new RoutesObjectModel(page, util);

    await util.expectRoute(RoutePath.main);
    await routes.main.gotoSettings();
    await routes.settings.gotoSplitTunnelingSettings();
  });

  test.afterAll(async () => {
    await page.close();
  });

  test.describe('Linux Split tunneling unsupported', () => {
    if (process.platform !== 'linux') {
      test.skip();
    }

    test.beforeAll(async () => {
      await util.ipc.linuxSplitTunneling.isSplitTunnelingSupported.handle(false);
      await util.ipc.linuxSplitTunneling.getApplications.handle(linuxApplicationsList);
    });

    test('App should show unsupported dialog when link in header is clicked', async () => {
      // Open the unsupported dialog
      await routes.splitTunnelingSettings.openUnsupportedDialog();
      const unsupportedText =
        routes.splitTunnelingSettings.getSplitTunnelingUnsupportedDialogText();
      await expect(unsupportedText).toBeVisible();

      // Close the unsupported dialog
      await routes.splitTunnelingSettings.closeUnsupportedDialog();
      await expect(unsupportedText).not.toBeVisible();
    });

    test('App list items should be shown even when split tunneling is unsupported', async () => {
      // Apps should be shown if split tunneling is unsupported
      const linuxApplications = routes.splitTunnelingSettings.getLinuxApplications();
      await expect(linuxApplications).toHaveCount(3);
    });

    test('App list items should show unsupported dialog when clicked', async () => {
      // Ensure clicking an application in the list makes the unsupported dialog visible
      const linuxApplications = routes.splitTunnelingSettings.getLinuxApplications();
      await linuxApplications.first().click();
      const unsupportedText =
        routes.splitTunnelingSettings.getSplitTunnelingUnsupportedDialogText();
      await expect(unsupportedText).toBeVisible();

      // Close the unsupported dialog
      await routes.splitTunnelingSettings.closeUnsupportedDialog();
      await expect(unsupportedText).not.toBeVisible();
    });
  });
});
