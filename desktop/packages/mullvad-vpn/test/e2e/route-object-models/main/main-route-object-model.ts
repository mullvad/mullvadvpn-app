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
    await this.utils.expectRoute(RoutePath.main);
  }

  async gotoSettings() {
    await this.selectors.settingsButton().click();
    await this.utils.expectRoute(RoutePath.settings);
  }

  async gotoSelectLocation() {
    await this.selectors.selectLocationButton().click();
    await this.utils.expectRoute(RoutePath.selectLocation);
  }

  async expandConnectionPanel() {
    await this.selectors.connectionPanelChevronButton().click();
  }

  getRelayHostname() {
    return this.selectors.relayHostname();
  }

  getInIp() {
    return this.selectors.inIpLabel();
  }

  getOutIps() {
    return this.selectors.outIpLabels();
  }
}
