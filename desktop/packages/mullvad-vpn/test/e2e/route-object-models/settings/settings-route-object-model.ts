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
    await this.utils.expectRoute(RoutePath.userInterfaceSettings);
  }

  async gotoVpnSettings() {
    await this.selectors.vpnSettingsButton().click();
    await this.utils.expectRoute(RoutePath.vpnSettings);
  }

  async gotoMultihopSettings() {
    await this.selectors.multihopSettingsButton().click();
    await this.utils.expectRoute(RoutePath.multihopSettings);
  }

  async gotoDaitaSettings() {
    await this.selectors.daitaSettingsButton().click();
    await this.utils.expectRoute(RoutePath.daitaSettings);
  }

  async gotoSplitTunnelingSettings() {
    await this.selectors.splitTunnelingSettingsButton().click();
    await this.utils.expectRoute(RoutePath.splitTunneling);
  }
}
