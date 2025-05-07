import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { RoutePath } from '../../../../src/renderer/lib/routes';
import { MockedTestUtils, startMockedApp } from '../mocked-utils';
import { createSelectors } from './helpers';

let page: Page;
let util: MockedTestUtils;
let selectors: ReturnType<typeof createSelectors>;

test.describe('User interface settings', () => {
  const startup = async () => {
    ({ page, util } = await startMockedApp());
    selectors = createSelectors(page);

    await util.waitForRoute(RoutePath.main);

    await page.click('button[aria-label="Settings"]');
    await util.waitForRoute(RoutePath.settings);
    await page.getByRole('button', { name: 'User interface settings' }).click();
    await util.waitForRoute(RoutePath.userInterfaceSettings);
  };

  test.beforeAll(async () => {
    await startup();
  });

  test.afterAll(async () => {
    await page.close();
  });

  test.describe('Select language', () => {
    ['Svenska', 'Deutsch', 'English', 'System default'].forEach((language) => {
      test(`Should change language to ${language}`, async () => {
        await selectors.languageButton().click();
        await util.waitForRoute(RoutePath.selectLanguage);

        await selectors.languageOption(language).click();
        await util.waitForRoute(RoutePath.userInterfaceSettings);

        await expect(selectors.languageButtonLabel(language)).toBeVisible();
      });
    });
  });
});
