import { expect, test } from '@playwright/test';
import { ElectronAppInfo } from 'electron-playwright-helpers';
import { Page } from 'playwright';

import { startApp } from './utils';

let appWindow: Page;
let appInfo: ElectronAppInfo;

test.beforeAll(async () => {
  const startAppResponse = await startApp();
  appInfo = startAppResponse.appInfo;
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

test('Validate status and header', async () => {
  const statusSpan = appWindow.locator('span[role="status"]');
  const header = appWindow.locator('header');

  await expect(statusSpan).toContainText('UNSECURED CONNECTION');
  let headerColor = await header.evaluate((el) => {
    return window.getComputedStyle(el).getPropertyValue('background-color');
  });
  expect(headerColor).toBe('rgb(227, 64, 57)');
  await appWindow.screenshot({ path: `e2e/screenshots/${appInfo.platform}/unsecured.png` });

  await appWindow.locator('text=Secure my connection').click();

  await expect(statusSpan).toContainText('SECURE CONNECTION');
  headerColor = await header.evaluate((el) => {
    return window.getComputedStyle(el).getPropertyValue('background-color');
  });
  expect(headerColor).toBe('rgb(68, 173, 77)');
  await appWindow.screenshot({ path: `e2e/screenshots/${appInfo.platform}/secure.png` });

  await appWindow.locator('text=Disconnect').click();
  await expect(statusSpan).toContainText('UNSECURED CONNECTION');
});
