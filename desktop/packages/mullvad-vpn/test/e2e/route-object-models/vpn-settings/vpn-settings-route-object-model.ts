import { Page } from 'playwright';

import { TestUtils } from '../../utils';
import { createSelectors } from './selectors';

export class VpnSettingsRouteObjectModel {
  readonly page: Page;
  readonly utils: TestUtils;
  readonly selectors: ReturnType<typeof createSelectors>;

  constructor(page: Page, utils: TestUtils) {
    this.page = page;
    this.utils = utils;
    this.selectors = createSelectors(page);
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

  async getAutoConnectSwitchState() {
    const autoConnectSwitch = this.getAutoConnectSwitch();
    return autoConnectSwitch.getAttribute('aria-checked');
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

  async getLaunchAppOnStartupSwitchState() {
    const launchAppOnStartupSwitch = this.getLaunchAppOnStartupSwitch();
    return launchAppOnStartupSwitch.getAttribute('aria-checked');
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
