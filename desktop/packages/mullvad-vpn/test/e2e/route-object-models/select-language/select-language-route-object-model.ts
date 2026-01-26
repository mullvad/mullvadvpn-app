import { Page } from 'playwright';

import { TestUtils } from '../../utils';
import { createSelectors } from './selectors';

export class SelectLanguageRouteObjectModel {
  readonly page: Page;
  readonly utils: TestUtils;
  readonly selectors: ReturnType<typeof createSelectors>;

  constructor(page: Page, util: TestUtils) {
    this.page = page;
    this.utils = util;
    this.selectors = createSelectors(page);
  }

  async selectLanguage(language: string) {
    await this.selectors.languageOption(language).click();
  }

  async goBack() {
    await this.utils.expectRouteChange(() => this.selectors.backButton().click());
  }
}
