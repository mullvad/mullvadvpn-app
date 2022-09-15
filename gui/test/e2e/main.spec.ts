import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { startApp } from './utils';

let appWindow: Page;

test.beforeAll(async () => {
  const startAppResponse = await startApp();
  appWindow = startAppResponse.appWindow;
});

test.afterAll(async () => {
  await appWindow.close();
});

test('Validate title', async () => {
  const title = await appWindow.title();
  expect(title).toBe('Mullvad VPN');
  await expect(appWindow.locator('header')).toBeVisible();
});
