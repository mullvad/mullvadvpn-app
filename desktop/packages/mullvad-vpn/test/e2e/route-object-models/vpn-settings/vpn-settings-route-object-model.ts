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

  async setAutoConnectSwitch(enabled: boolean) {
    const autoConnectSwitch = this.getAutoConnectSwitch();
    const ariaChecked = await autoConnectSwitch.getAttribute('aria-checked');

    if ((enabled && ariaChecked === 'false') || (!enabled && ariaChecked === 'true')) {
      await autoConnectSwitch.click();
    }
  }

  getLaunchAppOnStartupSwitch() {
    return this.selectors.launchAppOnStartupSwitch();
  }

  async setLaunchAppOnStartupSwitch(enabled: boolean) {
    const launchAppOnStartupSwitch = this.getLaunchAppOnStartupSwitch();
    const ariaChecked = await launchAppOnStartupSwitch.getAttribute('aria-checked');
    if ((enabled && ariaChecked === 'false') || (!enabled && ariaChecked === 'true')) {
      await launchAppOnStartupSwitch.click();
    }
  }

  getLanSwitch() {
    return this.selectors.lanSwitch();
  }

  async setLanSwitch(enabled: boolean) {
    const lanSwitch = this.getLanSwitch();
    const ariaChecked = await lanSwitch.getAttribute('aria-checked');

    if ((enabled && ariaChecked === 'false') || (!enabled && ariaChecked === 'true')) {
      await lanSwitch.click();
    }
  }
}
