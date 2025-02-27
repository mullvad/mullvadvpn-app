import { test } from '@playwright/test';
import { Page } from 'playwright';

import { RoutePath } from '../../../../src/renderer/lib/routes';
import { expectDisconnected } from '../../shared/tunnel-state';
import { TestUtils } from '../../utils';
import { startInstalledApp } from '../installed-utils';

// This test expects the daemon to be logged into an account that has time left and to be
// disconnected.

let page: Page;
let util: TestUtils;

test.beforeAll(async () => {
  ({ page, util } = await startInstalledApp());
  await util.waitForRoute(RoutePath.main);
});

test.afterAll(async () => {
  await page.close();
});

test('App should show disconnected tunnel state', async () => {
  await expectDisconnected(page);
});
