import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { startApp } from './utils';

let appWindow: Page;

test.beforeAll(async () => {
  const startAppResponse = await startApp();
  appWindow = startAppResponse.appWindow;
  await appWindow.click('button[aria-label="Settings"]');
});

test.afterAll(async () => {
  await appWindow.close();
});

test('Settings Page', async () => {
  const title = await appWindow.locator('h1');
  await expect(title).toContainText('Settings');

  const closeButton = await appWindow.locator('button[aria-label="Close"]');
  await expect(closeButton).toBeVisible();
});
