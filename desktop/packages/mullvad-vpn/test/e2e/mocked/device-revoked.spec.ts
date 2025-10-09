import { test } from '@playwright/test';
import { Page } from 'playwright';

import { RoutesObjectModel } from '../route-object-models';
import { MockedTestUtils, startMockedApp } from './mocked-utils';

let page: Page;
let util: MockedTestUtils;
let routes: RoutesObjectModel;

test.describe('Device revoked', () => {
  test.beforeAll(async () => {
    ({ page, util } = await startMockedApp());
    routes = new RoutesObjectModel(page, util);
    await routes.main.waitForRoute();
  });

  test.afterAll(async () => {
    await util?.closePage();
  });

  async function revokeDevice() {
    await util.ipc.account.device.notify({ type: 'revoked', deviceState: { type: 'revoked' } });
    await routes.deviceRevoked.waitForRoute();
  }

  test('Should navigate to device revoked view from main', async () => {
    await revokeDevice();
  });

  test.describe('Navigation from device revoked', () => {
    test.beforeEach(async () => {
      await revokeDevice();
    });

    test('Should navigate back to login view', async () => {
      await util.ipc.account.device.notify({
        type: 'logged out',
        deviceState: { type: 'logged out' },
      });
      await routes.login.waitForRoute();
    });

    test('Should navigate back to main view', async () => {
      await util.ipc.account.device.notify({
        type: 'logged in',
        deviceState: {
          type: 'logged in',
          accountAndDevice: {
            accountNumber: '1234123412341234',
            device: {
              id: '1',
              name: 'Test',
              created: new Date(),
            },
          },
        },
      });
      await routes.main.waitForRoute();
    });
  });
});
