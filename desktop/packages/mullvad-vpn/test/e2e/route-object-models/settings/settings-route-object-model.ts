import { Page } from 'playwright';

import { RoutePath } from '../../../../src/shared/routes';
import { MockedTestUtils } from '../../mocked/mocked-utils';
import { createSelectors } from './selectors';

export class SettingsRouteObjectModel {
  readonly page: Page;
  readonly utils: MockedTestUtils;
  readonly selectors: ReturnType<typeof createSelectors>;

  constructor(page: Page, utils: MockedTestUtils) {
    this.page = page;
    this.utils = utils;
    this.selectors = createSelectors(page);
  }

  async gotoUserInterfaceSettings() {
    await this.selectors.userInterfaceButton().click();
    await this.utils.waitForRoute(RoutePath.userInterfaceSettings);
  }
}
