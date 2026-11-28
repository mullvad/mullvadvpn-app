import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { RoutePath } from '../../../src/shared/routes';
import { MockedTestUtils, startMockedApp } from './mocked-utils';

let page: Page;
let util: MockedTestUtils;

test.beforeAll(async () => {
  ({ page, util } = await startMockedApp());
  await util.expectRoute(RoutePath.main);
});

test.afterAll(async () => {
  await util?.closePage();
});

test('WireGuard config button should be displayed correctly', async () => {
  const configButton = page.getByTestId('config-button');
  await expect(configButton).toBeVisible();
});

test('WireGuard config view should show the imported identity', async () => {
  await page.getByTestId('config-button').click();
  await util.expectRoute(RoutePath.wireguardConfig);

  await expect(page.getByTestId('wireguard-ipv4')).toContainText('10.64.0.2/32');
  await expect(page.getByTestId('wireguard-ipv6')).toContainText('fc00:bbbb:bbbb:bb01::1/128');
  await expect(page.getByTestId('wireguard-public-key')).toContainText(
    'bXlwdWJsaWNrZXlmb3J0ZXN0aW5nb25seTEyMzQ1Njc=',
  );

  await page.getByRole('button', { name: /(Back|Close)/ }).click();
  await util.expectRoute(RoutePath.main);
});

test('Settings Page', async () => {
  await page.click('button[aria-label="Settings"]');
  await util.expectRoute(RoutePath.settings);

  const title = page.locator('h1');
  await expect(title).toContainText('Settings');

  const closeButton = page.locator('button[aria-label="Close"]');
  await expect(closeButton).toBeVisible();
});
