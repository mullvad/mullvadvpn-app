import { Page } from 'playwright';

import { RoutePath } from '../../../../src/shared/routes';
import { TestUtils } from '../../utils';
import { createSelectors } from './selectors';

export class MainRouteObjectModel {
  readonly page: Page;
  readonly utils: TestUtils;
  readonly selectors: ReturnType<typeof createSelectors>;

  constructor(page: Page, util: TestUtils) {
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

  async gotoSelectLocation() {
    await this.selectors.selectLocationButton().click();
    await this.utils.waitForRoute(RoutePath.selectLocation);
  }

  async expandConnectionPanel() {
    await this.selectors.connectionPanelChevronButton().click();
  }

  getInIp() {
    return this.selectors.inIpLabel();
  }

  getInIpText() {
    return this.getInIp().innerText();
  }

  getOutIps() {
    return this.selectors.outIpLabels();
  }
}
