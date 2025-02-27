import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { RoutePath } from '../../../src/renderer/lib/routes';
import { TestUtils } from '../utils';
import { startMockedApp } from './mocked-utils';

let page: Page;
let util: TestUtils;

test.beforeAll(async () => {
  ({ page, util } = await startMockedApp());
  await util.waitForRoute(RoutePath.main);
});

test.afterAll(async () => {
  await page.close();
});

test('Validate title', async () => {
  const title = await page.title();
  expect(title).toBe('Mullvad VPN');
  await expect(page.locator('header')).toBeVisible();
});
