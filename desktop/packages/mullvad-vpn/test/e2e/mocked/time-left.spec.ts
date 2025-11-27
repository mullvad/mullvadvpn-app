import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { RoutesObjectModel } from '../route-object-models';
import { MockedTestUtils, startMockedApp } from './mocked-utils';

let page: Page;
let util: MockedTestUtils;
let routes: RoutesObjectModel;

const START_DATE = new Date('2025-01-01T13:37:00');
const ONE_DAY = 24 * 60 * 60 * 1_000;
const FUTURE_EXPIRY = {
  expiry: new Date(START_DATE.getTime() + 5 * ONE_DAY + 10_000).toISOString(),
};

test.describe('Time left label', () => {
  test.beforeAll(async () => {
    ({ page, util } = await startMockedApp());
    routes = new RoutesObjectModel(page, util);

    await routes.main.waitForRoute();
  });

  test.afterAll(async () => {
    await util?.closePage();
  });

  test.beforeEach(async () => {
    await page.clock.install({ time: START_DATE });
    await page.reload();
    await util.ipc.account[''].notify(FUTURE_EXPIRY);
  });

  test('Time left should update when time passes', async () => {
    const timeLeft = routes.main.selectors.timeLeftLabel();
    await expect(timeLeft).toHaveText(/5 days$/);

    await page.clock.runFor(ONE_DAY);
    await expect(timeLeft).toHaveText(/4 days$/);
  });
});
