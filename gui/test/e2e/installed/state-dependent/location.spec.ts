import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { TestUtils } from '../../utils';
import { startInstalledApp } from '../installed-utils';

// This test expects the daemon to be logged into an account that has time left.

let page: Page;
let util: TestUtils;

test.beforeAll(async () => {
  ({ page, util } = await startInstalledApp());
});

test.afterAll(async () => {
  await page.close();
});

test('App should have a country', async () => {
  const countryLabel = util.getByTestId('country');
  await expect(countryLabel).not.toBeEmpty();

  const cityLabel = util.getByTestId('city');
  const noCityLabel = await cityLabel.count() === 0;
  expect(noCityLabel).toBeTruthy();
});

