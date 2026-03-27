import { Page } from 'playwright';

import { RoutePath } from '../../../../src/shared/routes';
import { TestUtils } from '../../utils';
import { NavigationObjectModel } from '../navigation';
import { createSelectors } from './selectors';

export class VpnSettingsRouteObjectModel extends NavigationObjectModel {
  readonly selectors: ReturnType<typeof createSelectors>;

  constructor(
    public readonly page: Page,
    public readonly utils: TestUtils,
  ) {
    super(page, utils);
    this.selectors = createSelectors(page);
  }

  async gotoAntiCensorship() {
    await this.selectors.antiCensorship().click();
    await this.utils.expectRoute(RoutePath.antiCensorship);
  }

  getAutoConnectSwitch() {
    return this.selectors.autoConnectSwitch();
  }

  async setAutoConnectSwitch(enable: boolean) {
    const autoConnectSwitch = this.getAutoConnectSwitch();
    const checked = await autoConnectSwitch.isChecked();

    if ((enable && !checked) || (!enable && checked)) {
      await autoConnectSwitch.click();
    }
  }

  getLaunchAppOnStartupSwitch() {
    return this.selectors.launchAppOnStartupSwitch();
  }

  async setLaunchAppOnStartupSwitch(enable: boolean) {
    const launchAppOnStartupSwitch = this.getLaunchAppOnStartupSwitch();
    const checked = await launchAppOnStartupSwitch.isChecked();
    if ((enable && !checked) || (!enable && checked)) {
      await launchAppOnStartupSwitch.click();
    }
  }

  getLanSwitch() {
    return this.selectors.lanSwitch();
  }

  async setLanSwitch(enabled: boolean) {
    const lanSwitch = this.getLanSwitch();
    const checked = await lanSwitch.isChecked();

    if ((enabled && !checked) || (!enabled && checked)) {
      await lanSwitch.click();
    }
  }
}
