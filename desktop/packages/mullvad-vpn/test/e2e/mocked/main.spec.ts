import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { RoutePath } from '../../../src/shared/routes';
import { TestUtils } from '../utils';
import { startMockedApp } from './mocked-utils';

let page: Page;
let util: TestUtils;

test.beforeAll(async () => {
  ({ page, util } = await startMockedApp());
  await util.expectRoute(RoutePath.main);
});

test.afterAll(async () => {
  await util?.closePage();
});

test('Validate title', async () => {
  const title = await page.title();
  expect(title).toBe('Mullvad VPN');
  await expect(page.locator('header')).toBeVisible();
});
