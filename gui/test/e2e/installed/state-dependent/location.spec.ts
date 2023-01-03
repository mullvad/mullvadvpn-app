import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { startInstalledApp } from '../installed-utils';

// This test expects the daemon to be logged into an account that has time left.

let page: Page;

test.beforeAll(async () => {
  ({ page } = await startInstalledApp());
});

test.afterAll(async () => {
  await page.close();
});

test('App should have a country', async () => {
  const countryLabel = page.getByTestId('country');
  await expect(countryLabel).not.toBeEmpty();

  const cityLabel = page.getByTestId('city');
  const noCityLabel = await cityLabel.count() === 0;
  expect(noCityLabel).toBeTruthy();
});

