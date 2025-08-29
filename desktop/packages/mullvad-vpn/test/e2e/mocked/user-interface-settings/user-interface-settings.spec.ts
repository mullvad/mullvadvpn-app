import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { RoutesObjectModel } from '../../route-object-models';
import { MockedTestUtils, startMockedApp } from '../mocked-utils';

let page: Page;
let util: MockedTestUtils;
let routes: RoutesObjectModel;

test.describe('User interface settings', () => {
  const startup = async () => {
    ({ page, util } = await startMockedApp());
    routes = new RoutesObjectModel(page, util);

    await routes.main.waitForRoute();

    await routes.main.gotoSettings();
    await routes.settings.gotoUserInterfaceSettings();
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
        await routes.userInterfaceSettings.gotoSelectLanguage();
        await routes.selectLanguage.selectLanguage(language);

        await routes.userInterfaceSettings.waitForRoute();

        await expect(
          routes.userInterfaceSettings.getLocalizedLanguageButton(language),
        ).toBeVisible();
      });
    });
  });
});
