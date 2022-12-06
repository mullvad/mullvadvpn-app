import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { GetByTestId } from '../../utils';
import { startInstalledApp } from '../installed-utils';

let page: Page;
let getByTestId: GetByTestId;

test.beforeAll(async () => {
  ({ page, getByTestId } = await startInstalledApp());
});

test.afterAll(async () => {
  await page.close();
});

test('App should have a country', async () => {
  const countryLabel = getByTestId('country');
  await expect(countryLabel).not.toBeEmpty();

  const cityLabel = getByTestId('city');
  const noCityLabel = await cityLabel.count() === 0;
  expect(noCityLabel).toBeTruthy();
});

