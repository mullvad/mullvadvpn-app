import { expect, test } from '@playwright/test';
import { Page } from 'playwright';
import { RoutePath } from '../../../../src/renderer/lib/routes';
import { TestUtils } from '../../utils';
import { assertDisconnected } from '../../shared/tunnel-state';

import { startInstalledApp } from '../installed-utils';

// This test expects the daemon to be logged out and then log in.

let page: Page;
let util: TestUtils;

test.beforeAll(async () => {
  ({ page, util } = await startInstalledApp());
});

test.afterAll(async () => {
  await page.close();
});

// Disables timeout since it's handled by the rust test
test.setTimeout(0);

test('App should go from login view to main view when daemon logs in', async () => {
  expect(await util.currentRoute()).toEqual(RoutePath.login);

  // Waiting for the daemon to log in
  expect(await util.waitForNavigation()).toEqual(RoutePath.main);

  await assertDisconnected(page);
});
