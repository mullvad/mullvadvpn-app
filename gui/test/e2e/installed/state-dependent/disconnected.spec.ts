import { test } from '@playwright/test';
import { Page } from 'playwright';
import { assertDisconnected } from '../../shared/tunnel-state';

import { startInstalledApp } from '../installed-utils';

// This test expects the daemon to be logged into an account that has time left and to be
// disconnected.

let page: Page;

test.beforeAll(async () => {
  ({ page } = await startInstalledApp());
});

test.afterAll(async () => {
  await page.close();
});

test('App should show disconnected tunnel state', async () => {
  await assertDisconnected(page);
});
