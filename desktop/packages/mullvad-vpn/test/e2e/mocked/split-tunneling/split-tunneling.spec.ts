import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { RoutePath } from '../../../../src/shared/routes';
import { RoutesObjectModel } from '../../route-object-models';
import { MockedTestUtils, startMockedApp } from '../mocked-utils';

let page: Page;
let util: MockedTestUtils;
let routes: RoutesObjectModel;

test.describe('Select location', () => {
  test.beforeAll(async () => {
    ({ page, util } = await startMockedApp());
    routes = new RoutesObjectModel(page, util);

    await util.waitForRoute(RoutePath.main);
    await routes.main.gotoSettings();
    await routes.settings.gotoSplitTunnelingSettings();
  });

  test.afterAll(async () => {
    await page.close();
  });

  test.describe('Linux Split tunneling', () => {
    if (process.platform !== 'linux') {
      test.skip();
    }

    test('App should handle Linux split tunneling being unsupported', async () => {
      await util.ipc.linuxSplitTunneling.isSplitTunnelingSupported.handle(false);
      await util.ipc.linuxSplitTunneling.getApplications.handle([
        {
          absolutepath: '/app',
          exec: 'app',
          name: 'app',
          type: 'app',
          icon: '',
          warning: undefined,
        },
        {
          absolutepath: '/launches-elsewhere',
          exec: 'launches-elsewhere',
          name: 'launches-elsewhere',
          type: 'launches-elsewhere',
          icon: '',
          warning: 'launches-elsewhere',
        },
        {
          absolutepath: '/launches-in-existing-process',
          exec: 'launches-in-existing-process',
          name: 'launches-in-existing-process',
          type: 'launches-in-existing-process',
          icon: '',
          warning: 'launches-in-existing-process',
        },
      ]);

      // Apps should be shown even if split tunneling is unsupported
      const apps = routes.splitTunnelingSettings.getLinuxApplications();
      await expect(apps).toHaveCount(3);

      // Open the unsupported dialog
      await routes.splitTunnelingSettings.openUnsupportedDialog();

      // Ensure dialog contains unsupported text
      const unsupportedText = page.getByText(
        'To use Split tunneling, please change to a Linux kernel version that supports cgroup v1.',
      );
      await expect(unsupportedText).toBeVisible();

      // Close the unsupported dialog
      await routes.splitTunnelingSettings.closeUnsupportedDialog();
    });
  });
});
