import { test } from '@playwright/test';
import { Page } from 'playwright';

import { RoutesObjectModel } from '../route-object-models';
import { MockedTestUtils, startMockedApp } from './mocked-utils';

let page: Page;
let util: MockedTestUtils;
let routes: RoutesObjectModel;

test.describe('Too many devices', () => {
  test.beforeAll(async () => {
    ({ page, util } = await startMockedApp());
    routes = new RoutesObjectModel(page, util);
    await routes.main.waitForRoute();

    await util.ipc.account.device.notify({
      type: 'logged out',
      deviceState: { type: 'logged out' },
    });

    await routes.login.waitForRoute();
  });

  test.afterAll(async () => {
    await page.close();
  });

  test.describe('Navigation', () => {
    test('App should navigate to too many devices view', async () => {
      await util.ipc.account.login.handle({ type: 'error', error: 'too-many-devices' });
      await util.ipc.account.listDevices.handle([
        {
          id: '1',
          name: 'Device 1',
          created: new Date(),
        },
        {
          id: '2',
          name: 'Device 2',
          created: new Date(),
        },
      ]);

      await routes.login.fillAccountNumber('1234123412341234');
      await routes.login.loginByPressingEnter();

      await routes.tooManyDevices.waitForRoute();
    });

    test('App should navigate to main via login', async () => {
      await util.ipc.account.login.handle(undefined);

      await routes.tooManyDevices.waitForRoute();

      await routes.tooManyDevices.continue();
      await util.ipc.account.device.notify({
        type: 'logged in',
        deviceState: { type: 'logged in', accountAndDevice: { accountNumber: '1234123412341234' } },
      });
      await util.ipc.account[''].notify({ expiry: new Date(Date.now() + 60 * 1000).toISOString() });

      await routes.login.waitForRoute();
      await routes.main.waitForRoute();
    });
  });
});
