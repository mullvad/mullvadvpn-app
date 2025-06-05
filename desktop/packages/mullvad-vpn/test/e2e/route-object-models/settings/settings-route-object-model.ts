import { Page } from 'playwright';

import { RoutePath } from '../../../../src/shared/routes';
import { TestUtils } from '../../utils';
import { createSelectors } from './selectors';

export class SettingsRouteObjectModel {
  readonly page: Page;
  readonly utils: TestUtils;
  readonly selectors: ReturnType<typeof createSelectors>;

  constructor(page: Page, utils: TestUtils) {
    this.page = page;
    this.utils = utils;
    this.selectors = createSelectors(page);
  }

  async gotoUserInterfaceSettings() {
    await this.selectors.userInterfaceButton().click();
    await this.utils.waitForRoute(RoutePath.userInterfaceSettings);
  }

  async gotoVpnSettings() {
    await this.selectors.vpnSettingsButton().click();
    await this.utils.waitForRoute(RoutePath.vpnSettings);
  }
}
