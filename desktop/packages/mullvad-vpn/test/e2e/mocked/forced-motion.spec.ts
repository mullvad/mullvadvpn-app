import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { RoutesObjectModel } from '../route-object-models';
import { MockedTestUtils, startMockedApp } from './mocked-utils';

let page: Page;
let util: MockedTestUtils;
let routes: RoutesObjectModel;

test.describe('Transitions and animations', () => {
  test.skip(process.platform !== 'linux');

  test.beforeAll(async () => {
    ({ page, util } = await startMockedApp());
    routes = new RoutesObjectModel(page, util);
    await util.setReducedMotion('no-preference');

    await routes.main.waitForRoute();
  });

  test.afterAll(async () => {
    await util?.closePage();
  });

  test('Should navigate with transitions', async () => {
    await expectToTakeTime(() => routes.main.gotoSettings(), 450);
    await expectToTakeTime(() => routes.vpnSettings.goBack(), 450);

    await expectToTakeTime(async () => {
      await util.ipc.account.device.notify({
        type: 'logged out',
        deviceState: { type: 'logged out' },
      });
      await routes.login.waitForRoute();
    }, 450);
  });
});

async function expectToTakeTime(action: () => Promise<void> | void, minimumDuration: number) {
  const startTime = Date.now();
  await action();
  const duration = Date.now() - startTime;
  expect(duration).toBeGreaterThan(minimumDuration);
}
