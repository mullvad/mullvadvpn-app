import { test } from '@playwright/test';
import { Page } from 'playwright';

import { RoutesObjectModel } from '../route-object-models';
import { MockedTestUtils, startMockedApp } from './mocked-utils';

let page: Page;
let util: MockedTestUtils;
let routes: RoutesObjectModel;

const START_DATE = new Date('2025-01-01T13:37:00');

const CLOSE_TO_EXPIRY_DIFF = 60 * 60 * 1000;
const CLOSE_TO_EXPIRY_EXPIRY = {
  expiry: new Date(START_DATE.getTime() + CLOSE_TO_EXPIRY_DIFF).toISOString(),
};
const PASSED_EXPIRY = { expiry: new Date(START_DATE.getTime() - 60 * 1000).toISOString() };
const FUTURE_EXPIRY_DIFF = 30 * 24 * 60 * 60 * 1000;
const FUTURE_EXPIRY = {
  expiry: new Date(START_DATE.getTime() + FUTURE_EXPIRY_DIFF).toISOString(),
};

test.describe.configure({ mode: 'parallel' });

test.describe('Account expiry', () => {
  const startup = async () => {
    ({ page, util } = await startMockedApp());
    routes = new RoutesObjectModel(page, util);
  };

  test.beforeAll(async () => {
    await startup();
    await routes.main.waitForRoute();
  });

  test.afterAll(async () => {
    await page.close();
  });

  test.beforeEach(async () => {
    await page.clock.install({ time: START_DATE });
  });

  test('Should expire', async () => {
    await util.ipc.account[''].notify(CLOSE_TO_EXPIRY_EXPIRY);
    await routes.main.waitForRoute();
    await page.clock.fastForward(CLOSE_TO_EXPIRY_DIFF + 1);
    await routes.expired.waitForRoute();
  });

  function addTimeTests(newAccount: boolean) {
    test('Should respond to time added', async () => {
      await page.clock.fastForward('02:00');

      await Promise.all([
        routes.timeAdded.waitForRoute(),
        util.ipc.account[''].notify(FUTURE_EXPIRY),
      ]);

      await routes.timeAdded.gotoNext();

      if (newAccount) {
        await routes.setupFinished.waitForRoute();
        await routes.setupFinished.startUsingTheApp();
      } else {
        await routes.main.waitForRoute();
      }
    });

    test('Should redeem voucher', async () => {
      await page.clock.fastForward('20:00');

      const secondsAdded = FUTURE_EXPIRY_DIFF / 1000;

      await util.ipc.account.submitVoucher.handle({
        type: 'success',
        newExpiry: FUTURE_EXPIRY.expiry,
        secondsAdded,
      });

      await routes.expired.gotoRedeemVoucher();
      await routes.redeemVoucher.fillVoucherInput('1234-5678-90AB-CDEF');
      await page.clock.fastForward('02:00');

      await routes.redeemVoucher.redeemVoucher();
      await routes.voucherSuccess.waitForRoute(FUTURE_EXPIRY.expiry, secondsAdded);
      await routes.voucherSuccess.gotoNext();

      if (newAccount) {
        await routes.setupFinished.waitForRoute();
        await routes.setupFinished.startUsingTheApp();
      } else {
        await routes.main.waitForRoute();
      }
    });
  }

  test.describe('Has expired', () => {
    test.beforeEach(async () => {
      await util.ipc.account[''].notify(PASSED_EXPIRY);
      await routes.expired.waitForRoute();
    });

    addTimeTests(false);
  });

  test.describe('New account', () => {
    const logout = async () => {
      await util.ipc.account.device.notify({
        type: 'logged out',
        deviceState: { type: 'logged out' },
      });

      await routes.login.waitForRoute();
    };

    test.beforeEach(async () => {
      await logout();
      await util.ipc.account.create.handle('1234213412341234');
      await routes.login.createNewAccount();
      await util.ipc.account[''].notify({ expiry: START_DATE.toISOString() });
      await util.ipc.account.device.notify({
        type: 'logged in',
        deviceState: {
          type: 'logged in',
          accountAndDevice: {
            accountNumber: '1234213413241234',
            device: { id: '1', name: 'Successful Test', created: START_DATE },
          },
        },
      });
      await routes.expired.waitForRoute();
    });

    addTimeTests(true);
  });
});
