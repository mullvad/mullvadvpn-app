import { Page } from 'playwright';

import { MockedTestUtils } from '../../mocked/mocked-utils';
import { createSelectors } from './selectors';

export class SelectLanguageRouteObjectModel {
  readonly page: Page;
  readonly utils: MockedTestUtils;
  readonly selectors: ReturnType<typeof createSelectors>;

  constructor(page: Page, util: MockedTestUtils) {
    this.page = page;
    this.utils = util;
    this.selectors = createSelectors(page);
  }

  async selectLanguage(language: string) {
    await this.selectors.languageOption(language).click();
  }
}
