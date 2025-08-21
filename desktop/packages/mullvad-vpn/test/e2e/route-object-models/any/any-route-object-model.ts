import { Page } from 'playwright';

import { TestUtils } from '../../utils';
import { createSelectors } from './selectors';

export class AnyRouteObjectModel {
  readonly page: Page;
  readonly utils: TestUtils;
  readonly selectors: ReturnType<typeof createSelectors>;

  constructor(page: Page, utils: TestUtils) {
    this.page = page;
    this.utils = utils;
    this.selectors = createSelectors(page);
  }

  async goBack() {
    await this.selectors.backButton().click();
    await this.utils.waitForNextRoute();
  }

  async gotoRoot() {
    await this.page.press('body', 'Shift+Escape');
  }
}
