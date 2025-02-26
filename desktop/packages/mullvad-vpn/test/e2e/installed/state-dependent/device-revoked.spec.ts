import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { RoutePath } from '../../../../src/renderer/lib/routes';
import { TestUtils } from '../../utils';
import { startInstalledApp } from '../installed-utils';

// This test expects the daemon to be logged in to a revoked device.

let page: Page;
let util: TestUtils;

test.beforeAll(async () => {
  ({ page, util } = await startInstalledApp());
});

test.afterAll(async () => {
  await page.close();
});

test('App should fail to login', async () => {
  await util.waitForRoute(RoutePath.deviceRevoked);

  await expect(page.getByTestId('title')).toHaveText('Device is inactive');

  await page.getByText('Go to login').click();
  await util.waitForRoute(RoutePath.login);
});
