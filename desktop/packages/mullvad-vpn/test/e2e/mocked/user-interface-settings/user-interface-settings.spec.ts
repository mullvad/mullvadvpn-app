import { expect, test } from '@playwright/test';
import { Page } from 'playwright';

import { PageObjectModel } from '../../page-object-models';
import { MockedTestUtils, startMockedApp } from '../mocked-utils';

let page: Page;
let util: MockedTestUtils;
let pageObjectModel: PageObjectModel;

test.describe('User interface settings', () => {
  const startup = async () => {
    ({ page, util } = await startMockedApp());
    pageObjectModel = new PageObjectModel(page, util);

    await pageObjectModel.main.waitUntilOnPage();

    await pageObjectModel.main.gotoSettings();
    await pageObjectModel.settings.gotoUserInterfaceSettings();
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
        await pageObjectModel.userInterfaceSettings.gotoSelectLanguage();
        await pageObjectModel.selectLanguage.selectLanguage(language);

        await pageObjectModel.userInterfaceSettings.waitUntilOnPage();

        await expect(
          pageObjectModel.userInterfaceSettings.getLocalizedLanguageButton(language),
        ).toBeVisible();
      });
    });
  });
});
