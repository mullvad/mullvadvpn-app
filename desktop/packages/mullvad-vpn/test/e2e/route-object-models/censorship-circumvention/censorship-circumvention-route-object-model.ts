import { Page } from 'playwright';

import { RoutePath } from '../../../../src/shared/routes';
import { TestUtils } from '../../utils';
import { NavigationObjectModel } from '../navigation';
import { createSelectors } from './selectors';

export class CensorshipCircumventionRouteObjectModel extends NavigationObjectModel {
  readonly selectors: ReturnType<typeof createSelectors>;

  constructor(page: Page, utils: TestUtils) {
    super(page, utils);

    this.selectors = createSelectors(page);
  }

  async gotoUdpOverTcpSettings() {
    await this.selectors.udpOverTcpSettingsButton().click();
    await this.utils.expectRoute(RoutePath.udpOverTcp);
  }

  getAutomaticObfuscationOption() {
    return this.selectors.automaticOption();
  }

  async selectAutomaticObfuscation() {
    await this.getAutomaticObfuscationOption().click();
  }

  getUdpOverTcpOption() {
    return this.selectors.udpOverTcpOption();
  }

  async selectUdpOverTcp() {
    await this.getUdpOverTcpOption().click();
  }

  async selectShadowsocks() {
    await this.selectors.shadowsocksOption().click();
  }

  async gotoShadowSocksSettings() {
    await this.selectors.shadowsocksSettingsButton().click();
    await this.utils.expectRoute(RoutePath.shadowsocks);
  }
}
