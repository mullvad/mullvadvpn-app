import { Page } from 'playwright';

import { RoutePath } from '../../../../src/shared/routes';
import { MockedTestUtils } from '../../mocked/mocked-utils';
import { createSelectors } from './selectors';

export class UserInterfaceSettingsRouteObjectModel {
  readonly page: Page;
  readonly utils: MockedTestUtils;
  readonly selectors: ReturnType<typeof createSelectors>;

  constructor(page: Page, utils: MockedTestUtils) {
    this.page = page;
    this.utils = utils;
    this.selectors = createSelectors(page);
  }

  async gotoSelectLanguage() {
    await this.selectors.languageButton().click();
    await this.utils.waitForRoute(RoutePath.selectLanguage);
  }

  async waitForRoute() {
    await this.utils.waitForRoute(RoutePath.userInterfaceSettings);
  }

  getLocalizedLanguageButton(language: string) {
    return this.selectors.languageButtonLabel(language);
  }
}
