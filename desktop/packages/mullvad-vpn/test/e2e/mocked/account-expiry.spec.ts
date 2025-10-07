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
    await util?.closePage();
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

  // These tests verify that the renderer process will handle receiving the same expiry as
  // previously but at different system times, where for one system time the expiry is passed but
  // not for the other. This can happen if the system clock is changed.
  test.describe('Handle system clock changes', () => {
    test('Should move clock back', async () => {
      const expiry = {
        expiry: new Date('2025-04-03T13:00:00').toISOString(),
      };

      await page.clock.setSystemTime('2025-04-03T14:00:00');
      await util.ipc.account[''].notify(expiry);

      await routes.expired.waitForRoute();
      await page.clock.setSystemTime('2025-01-01T12:00');
      await util.ipc.account[''].notify(expiry);
      await routes.main.waitForRoute();
    });

    test('Should move clock forward', async () => {
      const expiry = {
        expiry: new Date('2025-04-03T13:00:00').toISOString(),
      };

      await page.clock.setSystemTime('2025-04-03T12:00:00');
      await util.ipc.account[''].notify(expiry);

      await routes.main.waitForRoute();
      await page.clock.setSystemTime('2025-04-04T12:00');
      await util.ipc.account[''].notify(expiry);
      await routes.expired.waitForRoute();
    });
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
      await page.clock.fastForward(1000);
      await routes.expired.waitForRoute();
    });

    addTimeTests(true);
  });
});
