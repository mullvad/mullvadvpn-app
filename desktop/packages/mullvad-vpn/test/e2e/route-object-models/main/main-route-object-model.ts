import { Page } from 'playwright';

import { RoutePath } from '../../../../src/shared/routes';
import { MockedTestUtils } from '../../mocked/mocked-utils';
import { createSelectors } from './selectors';

export class MainRouteObjectModel {
  readonly page: Page;
  readonly utils: MockedTestUtils;
  readonly selectors: ReturnType<typeof createSelectors>;

  constructor(page: Page, util: MockedTestUtils) {
    this.page = page;
    this.utils = util;
    this.selectors = createSelectors(page);
  }

  async waitForRoute() {
    await this.utils.waitForRoute(RoutePath.main);
  }

  async gotoSettings() {
    await this.selectors.settingsButton().click();
    await this.utils.waitForRoute(RoutePath.settings);
  }
}
